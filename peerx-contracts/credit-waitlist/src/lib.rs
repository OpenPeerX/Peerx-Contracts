#![no_std]
#![deny(missing_docs)]
//! Credit Waitlist Contract
//! 
//! A waitlist system for onboarding users to credit features.
mod storage;
mod queue;
mod onboarding;
mod admin;
mod events;

use soroban_sdk::{contract, contractimpl, Address, Env};
use storage::DataKey;

/// Credit Waitlist Contract.
/// 
/// This contract manages a waitlist for users requesting access to credit features.
#[contract]
pub struct CreditWaitlist;

#[contractimpl]
impl CreditWaitlist {
    /// Initialize the credit waitlist contract.
    /// 
    /// # Arguments
    /// * `env` - The Soroban environment
    /// * `admin` - The admin address for the contract
    /// * `batch_size` - The number of users to release in each batch
    /// 
    /// # Panics
    /// Panics if called by a non-admin user
    pub fn initialize(env: Env, admin: Address, batch_size: u32) {
        admin.require_auth();

        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage().instance().set(&DataKey::BatchSize, &batch_size);
        env.storage().instance().set(&DataKey::QueueStart, &0u32);
        env.storage().instance().set(&DataKey::QueueEnd, &0u32);
    }

    /// Join the credit waitlist.
    /// 
    /// # Arguments
    /// * `env` - The Soroban environment
    /// * `user` - The user joining the waitlist
    /// 
    /// # Events
    /// Emits a "joined" event when the user successfully joins
    pub fn join(env: Env, user: Address) {
        queue::join_queue(&env, user.clone());
        events::joined(&env, user);
    }

    /// Release a batch of users from the waitlist.
    /// 
    /// # Arguments
    /// * `env` - The Soroban environment
    /// * `admin` - The admin address
    /// 
    /// # Errors
    /// Returns an error if called by a non-admin user
    /// 
    /// # Panics
    /// Panics if called by a non-admin user
    pub fn release(env: Env, admin: Address) {
        admin::require_admin(&env, &admin);
        onboarding::release_batch(&env);
    }

    /// Mark a user as onboarded.
    /// 
    /// # Arguments
    /// * `env` - The Soroban environment
    /// * `user` - The user to mark as onboarded
    /// 
    /// # Events
    /// Emits an "onboarded" event
    pub fn onboard(env: Env, user: Address) {
        onboarding::mark_onboarded(&env, user.clone());
        events::onboarded(&env, user);
    }

    /// Get the waitlist status of a user.
    /// 
    /// # Arguments
    /// * `env` - The Soroban environment
    /// * `user` - The user to check
    /// 
    /// # Returns
    /// The waitlist status of the user
    /// 
    /// # Panics
    /// This function never panics
    pub fn get_status(env: Env, user: Address) -> storage::Status {
        env.storage()
            .instance()
            .get(&DataKey::Status(user))
            .unwrap()
    }
}