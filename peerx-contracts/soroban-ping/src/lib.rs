#![no_std]
#![deny(missing_docs)]
//! Soroban Ping Contract
//! 
//! A simple "hello world" contract for testing Soroban deployments and interactions.
use soroban_sdk::{contract, contractimpl, Env, String};

/// A simple ping/pong contract.
/// 
/// This contract responds to a `ping` call with a "pong" message.
#[contract]
pub struct PingContract;

#[contractimpl]
impl PingContract {
    /// Ping the contract and get a "pong" response.
    /// 
    /// # Arguments
    /// * `env` - The Soroban environment
    /// 
    /// # Returns
    /// The string "pong"
    /// 
    /// # Panics
    /// This function never panics
    pub fn ping(env: Env) -> String {
        String::from_str(&env, "pong")
    }
}

mod test;
