#![cfg(test)]

use super::*;
use soroban_sdk::{testutils::{Address as _, Ledger as _}, Address, Env, Symbol};

fn create_token_contract<'a>(env: &Env, admin: &Address) -> token::Client<'a> {
    token::Client::new(env, &env.register_stellar_asset_contract(admin.clone()))
}

#[test]
fn test_auction_flow() {
    let env = Env::default();
    env.mock_all_auths();

    // Setup actors
    let creator = Address::generate(&env);
    let bidder1 = Address::generate(&env);
    let bidder2 = Address::generate(&env);
    let token_admin = Address::generate(&env);

    // Deploy SEP-41 token
    let token = create_token_contract(&env, &token_admin);
    
    // Mint tokens to bidders
    let token_admin_client = token::StellarAssetClient::new(&env, &token.address);
    token_admin_client.mint(&bidder1, &1000);
    token_admin_client.mint(&bidder2, &1000);

    // Register our contract
    let contract_id = env.register_contract(None, NoLossAuctionContract);
    let client = NoLossAuctionContractClient::new(&env, &contract_id);

    // Initialize Auction
    let end_time = 100_u64;
    client.initialize(&creator, &token.address, &Symbol::new(&env, "ItemX"), &end_time, &100);

    // Getters assertions
    assert_eq!(client.get_creator(), creator);
    assert_eq!(client.get_token(), token.address);
    assert_eq!(client.get_item(), Symbol::new(&env, "ItemX"));
    assert_eq!(client.get_end_time(), end_time);
    assert_eq!(client.get_min_bid(), 100);
    assert_eq!(client.get_highest_bid(), 0);
    assert_eq!(client.get_highest_bidder(), None);
    assert_eq!(client.is_finalized(), false);

    // Bidder 1 bids 150
    client.bid(&bidder1, &150);
    assert_eq!(client.get_highest_bid(), 150);
    assert_eq!(client.get_highest_bidder(), Some(bidder1.clone()));
    assert_eq!(token.balance(&bidder1), 850);
    assert_eq!(token.balance(&contract_id), 150);

    // Bidder 2 bids 200 (Bidder 1 should be auto-refunded)
    client.bid(&bidder2, &200);
    assert_eq!(client.get_highest_bid(), 200);
    assert_eq!(client.get_highest_bidder(), Some(bidder2.clone()));
    // Bidder 1 got refunded 150
    assert_eq!(token.balance(&bidder1), 1000);
    // Bidder 2 paid 200
    assert_eq!(token.balance(&bidder2), 800);
    // Contract holds the active highest bid of 200
    assert_eq!(token.balance(&contract_id), 200);

    // Advance time to end_time
    env.ledger().set(soroban_sdk::testutils::LedgerInfo {
        timestamp: 101,
        protocol_version: 22,
        sequence_number: 1,
        network_id: [0; 32],
        base_reserve: 100,
        min_temp_entry_ttl: 1,
        min_persistent_entry_ttl: 1,
        max_entry_ttl: 1000,
    });

    // Finalize
    client.finalize();
    assert_eq!(client.is_finalized(), true);

    // Creator receives final highest bid (200)
    assert_eq!(token.balance(&creator), 200);
    assert_eq!(token.balance(&contract_id), 0);
}

#[test]
#[should_panic(expected = "cannot cancel after bids are placed")]
fn test_cannot_cancel_with_bids() {
    let env = Env::default();
    env.mock_all_auths();

    let creator = Address::generate(&env);
    let bidder = Address::generate(&env);
    let token_admin = Address::generate(&env);
    let token = create_token_contract(&env, &token_admin);
    let token_admin_client = token::StellarAssetClient::new(&env, &token.address);
    token_admin_client.mint(&bidder, &500);

    let contract_id = env.register_contract(None, NoLossAuctionContract);
    let client = NoLossAuctionContractClient::new(&env, &contract_id);

    let end_time = 100_u64;
    client.initialize(&creator, &token.address, &Symbol::new(&env, "ItemY"), &end_time, &100);
    client.bid(&bidder, &150);

    // Try to cancel (should panic)
    client.cancel();
}

#[test]
fn test_cancel_no_bids() {
    let env = Env::default();
    env.mock_all_auths();

    let creator = Address::generate(&env);
    let token_admin = Address::generate(&env);
    let token = create_token_contract(&env, &token_admin);

    let contract_id = env.register_contract(None, NoLossAuctionContract);
    let client = NoLossAuctionContractClient::new(&env, &contract_id);

    let end_time = 100_u64;
    client.initialize(&creator, &token.address, &Symbol::new(&env, "ItemZ"), &end_time, &100);
    
    // Cancel is allowed
    client.cancel();
    assert_eq!(client.is_finalized(), true);
}
