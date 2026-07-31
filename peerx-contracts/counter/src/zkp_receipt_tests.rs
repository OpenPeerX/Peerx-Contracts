#![cfg(test)]

use crate::zkp_types::ProofScheme;
use crate::zkp_verification::receipts::issue_receipt;
use crate::{CounterContract, CounterContractClient, Receipt, ZKPError, ZKProof};
use soroban_sdk::{Address, Bytes, Env};

fn setup() -> (Env, Address, CounterContractClient<'static>) {
    let env = Env::default();
    let contract_id = env.register_contract(None, CounterContract);
    let client = CounterContractClient::new(&env, &contract_id);
    (env, contract_id, client)
}

#[test]
fn private_tx_receipt_returns_proof_not_found_for_empty_hash() {
    let (env, _contract_id, client) = setup();
    let empty_hash = Bytes::new(&env);

    assert_eq!(
        client.try_private_tx_receipt(&empty_hash),
        Err(Ok(ZKPError::ProofNotFound))
    );
}

#[test]
fn private_tx_receipt_returns_proof_not_found_for_unknown_hash() {
    let (env, _contract_id, client) = setup();
    let unknown_hash = Bytes::from_array(&env, &[7u8; 32]);

    assert_eq!(
        client.try_private_tx_receipt(&unknown_hash),
        Err(Ok(ZKPError::ProofNotFound))
    );
}

#[test]
fn private_tx_receipt_returns_issued_receipt_with_all_fields() {
    let (env, contract_id, client) = setup();

    let tx_hash = Bytes::from_array(&env, &[1u8; 32]);
    let commitment = Bytes::from_array(&env, &[2u8; 32]);
    let witness_hash = Bytes::from_array(&env, &[3u8; 32]);
    let proof = ZKProof {
        proof_data: Bytes::from_array(&env, &[4u8; 32]),
        scheme: ProofScheme::ZkSnark,
    };

    env.as_contract(&contract_id, || {
        issue_receipt(
            &env,
            &tx_hash,
            commitment.clone(),
            witness_hash.clone(),
            proof.clone(),
        );
    });

    let receipt: Receipt = client.private_tx_receipt(&tx_hash);

    assert_eq!(receipt.commitment, commitment);
    assert_eq!(receipt.witness, witness_hash);
    assert_eq!(receipt.proof, proof);
}
