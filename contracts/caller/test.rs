#![cfg(test)]

use caller::interactor::Caller;
use erc20::erc20::{Erc20, Erc20Ref};
use ink::env::test;
use ink::env::test::DefaultAccounts;
use ink::U256;

fn default_accounts() -> DefaultAccounts<ink::env::DefaultEnvironment> {
    test::default_accounts::<ink::env::DefaultEnvironment>()
}

fn set_caller(caller: ink::primitives::AccountId) {
    ink::env::test::set_caller::<ink::env::DefaultEnvironment>(caller);
}

fn set_balance(account: ink::primitives::AccountId, balance: u128) {
    ink::env::test::set_account_balance::<ink::env::DefaultEnvironment>(account, balance);
}

#[ink::test]
fn constructor_works() {
    let accounts = default_accounts();
    let erc20 = Erc20::new(U256::from(1000));
    let erc20_ref = Erc20Ref::from_account_id(erc20.account_id());
    
    let caller = Caller::new(erc20_ref);
    assert_eq!(caller.get_token_address(), erc20.account_id());
}

#[ink::test]
fn token_total_supply_works() {
    let accounts = default_accounts();
    let total_supply = U256::from(1000);
    
    // Deploy ERC20
    let erc20 = Erc20::new(total_supply);
    let erc20_ref = Erc20Ref::from_account_id(erc20.account_id());
    
    // Deploy Caller
    let caller = Caller::new(erc20_ref);
    
    // Test total supply
    assert_eq!(caller.token_total_supply(), total_supply);
}

#[ink::test]
fn token_balance_of_works() {
    let accounts = default_accounts();
    let total_supply = U256::from(1000);
    
    // Deploy ERC20
    let erc20 = Erc20::new(total_supply);
    let erc20_ref = Erc20Ref::from_account_id(erc20.account_id());
    
    // Deploy Caller
    let caller = Caller::new(erc20_ref);
    
    // Test balance of deployer (should have all tokens)
    assert_eq!(caller.token_balance_of(accounts.alice), total_supply);
    
    // Test balance of other account (should be 0)
    assert_eq!(caller.token_balance_of(accounts.bob), U256::from(0));
}

#[ink::test]
fn token_transfer_works() {
    let accounts = default_accounts();
    let total_supply = U256::from(1000);
    let transfer_amount = U256::from(100);
    
    // Deploy ERC20
    let erc20 = Erc20::new(total_supply);
    let erc20_ref = Erc20Ref::from_account_id(erc20.account_id());
    
    // Deploy Caller
    let mut caller = Caller::new(erc20_ref);
    
    // Set caller to Alice (token holder)
    set_caller(accounts.alice);
    
    // Transfer tokens via caller
    let result = caller.token_transfer(accounts.bob, transfer_amount);
    assert!(result.is_ok());
    
    // Check balances
    assert_eq!(caller.token_balance_of(accounts.alice), total_supply - transfer_amount);
    assert_eq!(caller.token_balance_of(accounts.bob), transfer_amount);
}

#[ink::test]
fn token_approve_works() {
    let accounts = default_accounts();
    let total_supply = U256::from(1000);
    let approve_amount = U256::from(200);
    
    // Deploy ERC20
    let erc20 = Erc20::new(total_supply);
    let erc20_ref = Erc20Ref::from_account_id(erc20.account_id());
    
    // Deploy Caller
    let mut caller = Caller::new(erc20_ref);
    
    // Set caller to Alice (token holder)
    set_caller(accounts.alice);
    
    // Approve Bob to spend tokens
    let result = caller.token_approve(accounts.bob, approve_amount);
    assert!(result.is_ok());
    
    // Check allowance
    assert_eq!(caller.token_allowance(accounts.alice, accounts.bob), approve_amount);
}

#[ink::test]
fn token_transfer_from_works() {
    let accounts = default_accounts();
    let total_supply = U256::from(1000);
    let approve_amount = U256::from(200);
    let transfer_amount = U256::from(150);
    
    // Deploy ERC20
    let erc20 = Erc20::new(total_supply);
    let erc20_ref = Erc20Ref::from_account_id(erc20.account_id());
    
    // Deploy Caller
    let mut caller = Caller::new(erc20_ref);
    
    // Set caller to Alice (token holder)
    set_caller(accounts.alice);
    
    // Approve Bob to spend tokens
    let result = caller.token_approve(accounts.bob, approve_amount);
    assert!(result.is_ok());
    
    // Set caller to Bob (spender)
    set_caller(accounts.bob);
    
    // Transfer from Alice to Charlie via Bob
    let result = caller.token_transfer_from(accounts.alice, accounts.charlie, transfer_amount);
    assert!(result.is_ok());
    
    // Check balances
    assert_eq!(caller.token_balance_of(accounts.alice), total_supply - transfer_amount);
    assert_eq!(caller.token_balance_of(accounts.charlie), transfer_amount);
    
    // Check remaining allowance
    assert_eq!(caller.token_allowance(accounts.alice, accounts.bob), approve_amount - transfer_amount);
}

#[ink::test]
fn token_transfer_insufficient_balance_fails() {
    let accounts = default_accounts();
    let total_supply = U256::from(1000);
    let transfer_amount = U256::from(1500); // More than total supply
    
    // Deploy ERC20
    let erc20 = Erc20::new(total_supply);
    let erc20_ref = Erc20Ref::from_account_id(erc20.account_id());
    
    // Deploy Caller
    let mut caller = Caller::new(erc20_ref);
    
    // Set caller to Alice (token holder)
    set_caller(accounts.alice);
    
    // Try to transfer more than balance
    let result = caller.token_transfer(accounts.bob, transfer_amount);
    assert!(result.is_err());
    
    // Balances should remain unchanged
    assert_eq!(caller.token_balance_of(accounts.alice), total_supply);
    assert_eq!(caller.token_balance_of(accounts.bob), U256::from(0));
}

#[ink::test]
fn token_transfer_from_insufficient_allowance_fails() {
    let accounts = default_accounts();
    let total_supply = U256::from(1000);
    let approve_amount = U256::from(100);
    let transfer_amount = U256::from(150); // More than allowance
    
    // Deploy ERC20
    let erc20 = Erc20::new(total_supply);
    let erc20_ref = Erc20Ref::from_account_id(erc20.account_id());
    
    // Deploy Caller
    let mut caller = Caller::new(erc20_ref);
    
    // Set caller to Alice (token holder)
    set_caller(accounts.alice);
    
    // Approve Bob to spend tokens
    let result = caller.token_approve(accounts.bob, approve_amount);
    assert!(result.is_ok());
    
    // Set caller to Bob (spender)
    set_caller(accounts.bob);
    
    // Try to transfer more than allowance
    let result = caller.token_transfer_from(accounts.alice, accounts.charlie, transfer_amount);
    assert!(result.is_err());
    
    // Balances should remain unchanged
    assert_eq!(caller.token_balance_of(accounts.alice), total_supply);
    assert_eq!(caller.token_balance_of(accounts.charlie), U256::from(0));
}

#[ink::test]
fn multiple_transfers_work() {
    let accounts = default_accounts();
    let total_supply = U256::from(1000);
    let transfer1 = U256::from(100);
    let transfer2 = U256::from(200);
    
    // Deploy ERC20
    let erc20 = Erc20::new(total_supply);
    let erc20_ref = Erc20Ref::from_account_id(erc20.account_id());
    
    // Deploy Caller
    let mut caller = Caller::new(erc20_ref);
    
    // Set caller to Alice (token holder)
    set_caller(accounts.alice);
    
    // First transfer
    let result = caller.token_transfer(accounts.bob, transfer1);
    assert!(result.is_ok());
    
    // Second transfer
    let result = caller.token_transfer(accounts.charlie, transfer2);
    assert!(result.is_ok());
    
    // Check final balances
    assert_eq!(caller.token_balance_of(accounts.alice), total_supply - transfer1 - transfer2);
    assert_eq!(caller.token_balance_of(accounts.bob), transfer1);
    assert_eq!(caller.token_balance_of(accounts.charlie), transfer2);
}
