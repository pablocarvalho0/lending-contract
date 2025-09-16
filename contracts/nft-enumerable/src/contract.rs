// SPDX-License-Identifier: MIT
// Compatible with OpenZeppelin Stellar Soroban Contracts ^0.4.1

//! # Security
//!
//! For security issues, please contact: security@example.com
#![no_std]

use soroban_sdk::{Address, contract, contractimpl, Env, String, symbol_short, panic_with_error, Error};
use stellar_contract_utils::pausable::{self as pausable, Pausable};
use stellar_macros::{default_impl, when_not_paused};
use stellar_tokens::non_fungible::{
    Base, burnable::NonFungibleBurnable, enumerable::{NonFungibleEnumerable, Enumerable},
    NonFungibleToken
};

#[contract]
pub struct LendingNFT;

#[contractimpl]
impl LendingNFT {
    pub fn __constructor(e: &Env, owner: Address) {
        let uri = String::from_str(e, "www.lendingnft.com");
        let name = String::from_str(e, "LendingNFT");
        let symbol = String::from_str(e, "LNF");
        Base::set_metadata(e, uri, name, symbol);
        e.storage().instance().set(&symbol_short!("owner"), &owner);
    }

    #[when_not_paused]
    pub fn mint(e: &Env, to: Address, token_id: u32, caller: Address) {
        let owner = e.storage().instance().get(&symbol_short!("owner"))
            .unwrap_or_else(|| panic_with_error!(e, Error::from_contract_error(1)));
        if caller != owner {
            panic_with_error!(e, Error::from_contract_error(2));
        }
        Enumerable::non_sequential_mint(e, &to, token_id);
    }

    // ===== LENDING FUNCTIONS =====

    /// Create a loan using NFT as collateral
    pub fn create_loan(
        e: &Env,
        borrower: Address,
        token_id: u32,
        amount: i128,
        interest_rate: u32,
        duration_days: u32,
        _caller: Address
    ) -> u32 {
        // Check if caller owns the NFT
        if Enumerable::owner_of(e, token_id) != borrower {
            panic_with_error!(e, Error::from_contract_error(3));
        }

        // Check if NFT is already used as collateral
        if Self::is_collateral(e, token_id) {
            panic_with_error!(e, Error::from_contract_error(4));
        }

        let loan_id = Self::get_next_loan_id(e);
        
        // Store loan data
        e.storage().instance().set(&symbol_short!("borrower"), &borrower);
        e.storage().instance().set(&symbol_short!("amount"), &amount);
        e.storage().instance().set(&symbol_short!("rate"), &interest_rate);
        e.storage().instance().set(&symbol_short!("duration"), &duration_days);
        e.storage().instance().set(&symbol_short!("created"), &e.ledger().timestamp());
        e.storage().instance().set(&symbol_short!("status"), &0u32); // Active
        e.storage().instance().set(&symbol_short!("repaid"), &0i128);
        
        // Mark token as collateral
        e.storage().instance().set(&symbol_short!("collat"), &token_id);
        
        Self::increment_next_loan_id(e);
        loan_id
    }

    /// Repay a loan
    pub fn repay_loan(
        e: &Env,
        _loan_id: u32,
        amount: i128,
        caller: Address
    ) {
        let borrower: Address = e.storage().instance().get(&symbol_short!("borrower"))
            .unwrap_or_else(|| panic_with_error!(e, Error::from_contract_error(5)));
        
        if borrower != caller {
            panic_with_error!(e, Error::from_contract_error(6));
        }

        let status = e.storage().instance().get(&symbol_short!("status")).unwrap_or(1u32);
        if status != 0 {
            panic_with_error!(e, Error::from_contract_error(7));
        }

        // Simple repayment - just update repaid amount
        let repaid = e.storage().instance().get(&symbol_short!("repaid")).unwrap_or(0i128);
        let new_repaid = repaid + amount;
        e.storage().instance().set(&symbol_short!("repaid"), &new_repaid);

        // If fully repaid, mark as repaid
        let loan_amount = e.storage().instance().get(&symbol_short!("amount")).unwrap_or(0i128);
        if new_repaid >= loan_amount {
            e.storage().instance().set(&symbol_short!("status"), &1u32);
            e.storage().instance().set(&symbol_short!("collat"), &0u32);
        }
    }

    /// Get loan information
    pub fn get_loan_info(e: &Env, _loan_id: u32) -> (Address, i128, u32, u32, u64, u32, i128) {
        let borrower: Address = e.storage().instance().get(&symbol_short!("borrower"))
            .unwrap_or_else(|| panic_with_error!(e, Error::from_contract_error(8)));
        let amount = e.storage().instance().get(&symbol_short!("amount")).unwrap_or(0i128);
        let interest_rate = e.storage().instance().get(&symbol_short!("rate")).unwrap_or(0u32);
        let duration = e.storage().instance().get(&symbol_short!("duration")).unwrap_or(0u32);
        let created_at = e.storage().instance().get(&symbol_short!("created")).unwrap_or(0u64);
        let status = e.storage().instance().get(&symbol_short!("status")).unwrap_or(1u32);
        let repaid = e.storage().instance().get(&symbol_short!("repaid")).unwrap_or(0i128);
        
        (borrower, amount, interest_rate, duration, created_at, status, repaid)
    }

    /// Check if NFT is used as collateral
    pub fn is_collateral(e: &Env, token_id: u32) -> bool {
        e.storage().instance().get(&symbol_short!("collat")).unwrap_or(0u32) == token_id
    }

    // ===== HELPER FUNCTIONS =====

    fn get_next_loan_id(e: &Env) -> u32 {
        e.storage().instance().get(&symbol_short!("next_id")).unwrap_or(1)
    }

    fn increment_next_loan_id(e: &Env) {
        let current = Self::get_next_loan_id(e);
        e.storage().instance().set(&symbol_short!("next_id"), &(current + 1));
    }
}

// ============ NFT IMPLEMENTATIONS ============

#[default_impl]
#[contractimpl]
impl NonFungibleToken for LendingNFT {
    type ContractType = Enumerable;

    #[when_not_paused]
    fn transfer(e: &Env, from: Address, to: Address, token_id: u32) {
        Self::ContractType::transfer(e, &from, &to, token_id);
    }

    #[when_not_paused]
    fn transfer_from(e: &Env, spender: Address, from: Address, to: Address, token_id: u32) {
        Self::ContractType::transfer_from(e, &spender, &from, &to, token_id);
    }
}

#[contractimpl]
impl NonFungibleBurnable for LendingNFT {
    #[when_not_paused]
    fn burn(e: &Env, from: Address, token_id: u32) {
        Self::ContractType::burn(e, &from, token_id);
    }

    #[when_not_paused]
    fn burn_from(e: &Env, spender: Address, from: Address, token_id: u32) {
        Self::ContractType::burn_from(e, &spender, &from, token_id);
    }
}

#[default_impl]
#[contractimpl]
impl NonFungibleEnumerable for LendingNFT {}

#[contractimpl]
impl Pausable for LendingNFT {
    fn paused(e: &Env) -> bool {
        pausable::paused(e)
    }

    fn pause(e: &Env, caller: Address) {
        let owner = e.storage().instance().get(&symbol_short!("owner"))
            .unwrap_or_else(|| panic_with_error!(e, Error::from_contract_error(1)));
        if caller != owner {
            panic_with_error!(e, Error::from_contract_error(2));
        }
        pausable::pause(e);
    }

    fn unpause(e: &Env, caller: Address) {
        let owner = e.storage().instance().get(&symbol_short!("owner"))
            .unwrap_or_else(|| panic_with_error!(e, Error::from_contract_error(1)));
        if caller != owner {
            panic_with_error!(e, Error::from_contract_error(2));
        }
        pausable::unpause(e);
    }
}