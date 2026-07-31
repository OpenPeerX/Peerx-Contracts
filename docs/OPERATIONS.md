# PeerX Metrics Exporter — Operations

## Overview

`peerx-metrics-exporter` is a lightweight sidecar binary that polls
the PeerX Soroban contract's `get_cache_stats` read-only entry point
and exposes the result as Prometheus-compatible metrics on an HTTP
endpoint.

## Environment Variables

| Variable | Default | Description |
|---|---|---|
| `PEERX_CONTRACT_ID` | *(required)* | Stellar contract address to query |
| `STELLAR_RPC_URL` | `https://soroban-rpc.mainnet.stellar.org` | Soroban RPC endpoint |
| `POLL_INTERVAL_SECS` | `15` | Seconds between on-chain polls |
| `LISTEN_PORT` | `9090` | HTTP port for `/metrics` and `/health` |

## Endpoints

| Path | Method | Description |
|---|---|---|
| `/metrics` | GET | Prometheus text exposition format |
| `/health` | GET | `{"status":"ok"}` liveness probe |

## Build

```bash
cargo build --release -p peerx-metrics-exporter
```

Binary is produced at `target/release/peerx-metrics-exporter`.

## Systemd Unit

Create `/etc/systemd/system/peerx-metrics-exporter.service`:

```ini
[Unit]
Description=PeerX Cache Metrics Exporter
After=network-online.target
Wants=network-online.target

[Service]
Type=simple
User=peerx
Group=peerx
Environment=PEERX_CONTRACT_ID=CDLZ...YOUR_CONTRACT_ADDRESS
Environment=STELLAR_RPC_URL=https://soroban-rpc.mainnet.stellar.org
Environment=POLL_INTERVAL_SECS=15
Environment=LISTEN_PORT=9090
ExecStart=/usr/local/bin/peerx-metrics-exporter
Restart=always
RestartSec=5
StandardOutput=journal
StandardError=journal

[Install]
WantedBy=multi-user.target
```

Then enable and start:

```bash
sudo systemctl daemon-reload
sudo systemctl enable --now peerx-metrics-exporter
sudo journalctl -u peerx-metrics-exporter -f
```

## Prometheus Scrape Config

Add to `prometheus.yml`:

```yaml
scrape_configs:
  - job_name: peerx-metrics
    scrape_interval: 15s
    static_configs:
      - targets: ['localhost:9090']
```

## Grafana Dashboard

Import `dashboards/cache.json` into Grafana:

1. Navigate to **Dashboards → Import**
2. Upload `cache.json` or paste its contents
3. Select your Prometheus datasource
4. Click **Import**

The dashboard shows:
- **Cache Hit Ratio** (stat panel, basis points)
- **Cache Hits vs Misses** (time series)
- **Poll Errors** (stat panel)
- **Last Poll Timestamp** (stat panel)

## Docker (alternative)

```bash
docker build -t peerx-metrics-exporter bin/peerx-metrics-exporter/
docker run -d \
  -e PEERX_CONTRACT_ID=CDLZ... \
  -e STELLAR_RPC_URL=https://soroban-rpc.mainnet.stellar.org \
  -p 9090:9090 \
  peerx-metrics-exporter
```
