use prometheus::{Encoder, IntGauge, Registry, TextEncoder};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use std::sync::Arc;
use tracing::{error, info};

/// PeerX contract address (set via environment or config)
fn contract_id() -> String {
    std::env::var("PEERX_CONTRACT_ID").unwrap_or_else(|_| {
        panic!("PEERX_CONTRACT_ID environment variable must be set")
    })
}

/// Stellar RPC node URL
fn rpc_url() -> String {
    std::env::var("STELLAR_RPC_URL")
        .unwrap_or_else(|_| "https://soroban-rpc.mainnet.stellar.org".to_string())
}

/// How often to poll the on-chain contract (seconds).
fn poll_interval_secs() -> u64 {
    std::env::var("POLL_INTERVAL_SECS")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(15)
}

/// HTTP listen port.
fn listen_port() -> u16 {
    std::env::var("LISTEN_PORT")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(9090)
}

// ── Stellar RPC request / response types ────────────────────────────────────

#[derive(Serialize)]
struct RpcRequest {
    jsonrpc: String,
    id: u32,
    method: String,
    params: serde_json::Value,
}

#[derive(Deserialize)]
struct RpcResponse {
    result: serde_json::Value,
}

// ── Prometheus metrics ──────────────────────────────────────────────────────

struct Metrics {
    cache_hits: IntGauge,
    cache_misses: IntGauge,
    cache_hit_ratio_bps: IntGauge,
    poll_errors_total: IntGauge,
    last_poll_timestamp: IntGauge,
}

impl Metrics {
    fn new(registry: &Registry) -> Self {
        let cache_hits = IntGauge::new("peerx_cache_hits_total", "Total cache hits").unwrap();
        let cache_misses =
            IntGauge::new("peerx_cache_misses_total", "Total cache misses").unwrap();
        let cache_hit_ratio_bps =
            IntGauge::new("peerx_cache_hit_ratio_bps", "Cache hit ratio in basis points")
                .unwrap();
        let poll_errors_total =
            IntGauge::new("peerx_poll_errors_total", "Total RPC poll errors").unwrap();
        let last_poll_timestamp =
            IntGauge::new("peerx_last_poll_timestamp", "Unix timestamp of last successful poll")
                .unwrap();

        registry.register(Box::new(cache_hits.clone())).unwrap();
        registry.register(Box::new(cache_misses.clone())).unwrap();
        registry.register(Box::new(cache_hit_ratio_bps.clone())).unwrap();
        registry.register(Box::new(poll_errors_total.clone())).unwrap();
        registry.register(Box::new(last_poll_timestamp.clone())).unwrap();

        Self {
            cache_hits,
            cache_misses,
            cache_hit_ratio_bps,
            poll_errors_total,
            last_poll_timestamp,
        }
    }
}

// ── RPC helper ──────────────────────────────────────────────────────────────

async fn invoke_read_only(
    client: &Client,
    rpc: &str,
    contract: &str,
    function: &str,
) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    let request = RpcRequest {
        jsonrpc: "2.0".to_string(),
        id: 1,
        method: "simulateTransaction".to_string(),
        params: serde_json::json!({
            "transaction": {
                "sourceAccount": contract,
                "fee": 100,
                "seqNum": "1",
                "operations": [{
                    "type": "invokeContract",
                    "contract": contract,
                    "function": function,
                    "args": []
                }]
            }
        }),
    };

    let resp = client.post(rpc).json(&request).send().await?;
    let body: RpcResponse = resp.json().await?;
    Ok(body.result)
}

// ── Polling loop ────────────────────────────────────────────────────────────

async fn poll_cache_stats(
    client: Client,
    rpc: String,
    contract: String,
    metrics: Arc<Metrics>,
) {
    loop {
        match invoke_read_only(&client, &rpc, &contract, "get_cache_stats").await {
            Ok(result) => {
                // Parse the result: get_cache_stats returns (hits: u64, misses: u64, ratio_bps: u32)
                if let Some(arr) = result.as_array() {
                    if arr.len() >= 3 {
                        let hits = arr[0].as_i64().unwrap_or(0);
                        let misses = arr[1].as_i64().unwrap_or(0);
                        let ratio_bps = arr[2].as_i64().unwrap_or(0);

                        metrics.cache_hits.set(hits);
                        metrics.cache_misses.set(misses);
                        metrics.cache_hit_ratio_bps.set(ratio_bps);
                        metrics.last_poll_timestamp.set(
                            std::time::SystemTime::now()
                                .duration_since(std::time::UNIX_EPOCH)
                                .unwrap_or_default()
                                .as_secs() as i64,
                        );

                        info!(
                            hits, misses, ratio_bps,
                            "Cache stats updated"
                        );
                    }
                }
            }
            Err(e) => {
                error!("Failed to poll cache stats: {}", e);
                metrics.poll_errors_total.inc();
            }
        }

        tokio::time::sleep(std::time::Duration::from_secs(poll_interval_secs())).await;
    }
}

// ── HTTP server ─────────────────────────────────────────────────────────────

async fn metrics_handler(
    registry: Arc<Registry>,
) -> Result<impl warp::Reply, warp::Rejection> {
    let encoder = TextEncoder::new();
    let metric_families = registry.gather();
    let mut buffer = Vec::new();
    encoder.encode(&metric_families, &mut buffer).unwrap();

    Ok(warp::reply::with_header(
        buffer,
        "Content-Type",
        encoder.format_type(),
    ))
}

async fn health_handler() -> Result<impl warp::Reply, warp::Rejection> {
    Ok(warp::reply::json(&serde_json::json!({
        "status": "ok",
        "service": "peerx-metrics-exporter"
    })))
}

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();

    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info".into()),
        )
        .init();

    let rpc = rpc_url();
    let contract = contract_id();
    let port = listen_port();

    info!(%rpc, %contract, %port, "Starting PeerX metrics exporter");

    // Prometheus registry
    let registry = Arc::new(Registry::new());
    let metrics = Arc::new(Metrics::new(&registry));

    // Spawn the polling task
    let poll_client = Client::new();
    let poll_rpc = rpc.clone();
    let poll_contract = contract.clone();
    let poll_metrics = metrics.clone();
    tokio::spawn(async move {
        poll_cache_stats(poll_client, poll_rpc, poll_contract, poll_metrics).await;
    });

    // HTTP routes
    let reg = registry.clone();
    let metrics_route = warp::path("metrics")
        .and(warp::get())
        .and(warp::any().map(move || reg.clone()))
        .and_then(metrics_handler);

    let health_route = warp::path("health")
        .and(warp::get())
        .and_then(health_handler);

    let routes = metrics_route.or(health_route);

    let addr: SocketAddr = ([0, 0, 0, 0], port).into();
    info!(%addr, "HTTP server listening");
    warp::serve(routes).run(addr).await;
}
