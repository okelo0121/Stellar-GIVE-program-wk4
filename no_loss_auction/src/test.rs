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

    let creator = Address::generate(&env);
    let bidder1 = Address::generate(&env);
    let bidder2 = Address::generate(&env);
    let token_admin = Address::generate(&env);

    let token = create_token_contract(&env, &token_admin);
    
    let token_admin_client = token::StellarAssetClient::new(&env, &token.address);
    token_admin_client.mint(&bidder1, &1000);
    token_admin_client.mint(&bidder2, &1000);

    let contract_id = env.register_contract(None, NoLossAuctionContract);
    let client = NoLossAuctionContractClient::new(&env, &contract_id);

    let end_time = 100_u64;
    client.initialize(&creator, &token.address, &Symbol::new(&env, "ItemX"), &end_time, &100);

    // Assert initial state via the single struct getter
    let state = client.get_state().unwrap();
    assert_eq!(state.creator, creator);
    assert_eq!(state.token, token.address);
    assert_eq!(state.item, Symbol::new(&env, "ItemX"));
    assert_eq!(state.end_time, end_time);
    assert_eq!(state.min_bid, 100);
    assert_eq!(state.highest_bid, 0);
    assert_eq!(state.highest_bidder, None);
    assert_eq!(state.finalized, false);

    // First bid
    client.bid(&bidder1, &150);
    let state = client.get_state().unwrap();
    assert_eq!(state.highest_bid, 150);
    assert_eq!(state.highest_bidder, Some(bidder1.clone()));
    assert_eq!(token.balance(&bidder1), 850);
    assert_eq!(token.balance(&contract_id), 150);

    // Outbid and refund verification
    client.bid(&bidder2, &200);
    let state = client.get_state().unwrap();
    assert_eq!(state.highest_bid, 200);
    assert_eq!(state.highest_bidder, Some(bidder2.clone()));
    assert_eq!(token.balance(&bidder1), 1000); 
    assert_eq!(token.balance(&bidder2), 800);
    assert_eq!(token.balance(&contract_id), 200);

    // Wrap time forward
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

    client.finalize();
    let state = client.get_state().unwrap();
    assert_eq!(state.finalized, true);

    assert_eq!(token.balance(&creator), 200);
    assert_eq!(token.balance(&contract_id), 0);
}

#[test]
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

    // Using try_cancel avoids string matching and lets us inspect the exact error variant
    let result = client.try_cancel();
    assert!(result.is_err());
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
    
    client.cancel();
    let state = client.get_state().unwrap();
    assert_eq!(state.finalized, true);
}
