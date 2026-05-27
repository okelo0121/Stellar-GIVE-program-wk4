#![no_std]
use soroban_sdk::{contract, contractimpl, contracttype, token, Address, Env, Symbol};

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DataKey {
    Creator,      // Address
    Token,        // Address (SEP-41 Token used for bidding)
    ItemName,     // Symbol
    EndTime,      // u64 (Timestamp in seconds)
    MinBid,       // i128
    HighestBid,   // i128
    HighestBidder,// Address (optional/none if no bids)
    Finalized,    // bool
}

#[contract]
pub struct NoLossAuctionContract;

#[contractimpl]
impl NoLossAuctionContract {
    /// Initialize a new auction
    pub fn initialize(
        env: Env,
        creator: Address,
        token: Address,
        item: Symbol,
        end_time: u64,
        min_bid: i128,
    ) {
        if env.storage().instance().has(&DataKey::Creator) {
            panic!("already initialized");
        }
        if end_time <= env.ledger().timestamp() {
            panic!("end time must be in the future");
        }
        if min_bid <= 0 {
            panic!("min bid must be positive");
        }

        env.storage().instance().set(&DataKey::Creator, &creator);
        env.storage().instance().set(&DataKey::Token, &token);
        env.storage().instance().set(&DataKey::ItemName, &item);
        env.storage().instance().set(&DataKey::EndTime, &end_time);
        env.storage().instance().set(&DataKey::MinBid, &min_bid);
        env.storage().instance().set(&DataKey::HighestBid, &0_i128);
        env.storage().instance().set(&DataKey::Finalized, &false);
    }

    /// Place a bid
    pub fn bid(env: Env, bidder: Address, amount: i128) {
        bidder.require_auth();

        let finalized: bool = env.storage().instance().get(&DataKey::Finalized).unwrap_or(false);
        if finalized {
            panic!("auction is finalized");
        }

        let end_time: u64 = env.storage().instance().get(&DataKey::EndTime).unwrap();
        if env.ledger().timestamp() >= end_time {
            panic!("auction already ended");
        }

        let min_bid: i128 = env.storage().instance().get(&DataKey::MinBid).unwrap();
        let highest_bid: i128 = env.storage().instance().get(&DataKey::HighestBid).unwrap();

        let required_min = if highest_bid == 0 { min_bid } else { highest_bid + 1 };
        if amount < required_min {
            panic!("bid too low");
        }

        let token_addr: Address = env.storage().instance().get(&DataKey::Token).unwrap();
        let token_client = token::Client::new(&env, &token_addr);

        // Refund the previous highest bidder if there was one
        if env.storage().instance().has(&DataKey::HighestBidder) {
            let prev_bidder: Address = env.storage().instance().get(&DataKey::HighestBidder).unwrap();
            let prev_amount: i128 = highest_bid;
            
            // Return funds from the contract to the previous bidder
            token_client.transfer(&env.current_contract_address(), &prev_bidder, &prev_amount);
        }

        // Transfer new bid funds to this contract
        token_client.transfer(&bidder, &env.current_contract_address(), &amount);

        // Update state
        env.storage().instance().set(&DataKey::HighestBidder, &bidder);
        env.storage().instance().set(&DataKey::HighestBid, &amount);
    }

    /// Finalize the auction (can be called by anyone after the deadline)
    pub fn finalize(env: Env) {
        let finalized: bool = env.storage().instance().get(&DataKey::Finalized).unwrap_or(false);
        if finalized {
            panic!("already finalized");
        }

        let end_time: u64 = env.storage().instance().get(&DataKey::EndTime).unwrap();
        if env.ledger().timestamp() < end_time {
            panic!("deadline not passed yet");
        }

        let creator: Address = env.storage().instance().get(&DataKey::Creator).unwrap();
        let highest_bid: i128 = env.storage().instance().get(&DataKey::HighestBid).unwrap();

        if highest_bid > 0 {
            // Transfer highest bid to the creator (winner gets the item off-chain / bid goes to creator)
            let token_addr: Address = env.storage().instance().get(&DataKey::Token).unwrap();
            let token_client = token::Client::new(&env, &token_addr);
            token_client.transfer(&env.current_contract_address(), &creator, &highest_bid);
        }

        env.storage().instance().set(&DataKey::Finalized, &true);
    }

    /// Cancel the auction (only by creator, and only if no bids have been placed)
    pub fn cancel(env: Env) {
        let creator: Address = env.storage().instance().get(&DataKey::Creator).unwrap();
        creator.require_auth();

        let finalized: bool = env.storage().instance().get(&DataKey::Finalized).unwrap_or(false);
        if finalized {
            panic!("already finalized");
        }

        if env.storage().instance().has(&DataKey::HighestBidder) {
            panic!("cannot cancel after bids are placed");
        }

        env.storage().instance().set(&DataKey::Finalized, &true);
    }

    // --- Getter functions for frontend integration ---

    pub fn get_creator(env: Env) -> Address {
        env.storage().instance().get(&DataKey::Creator).unwrap()
    }

    pub fn get_token(env: Env) -> Address {
        env.storage().instance().get(&DataKey::Token).unwrap()
    }

    pub fn get_item(env: Env) -> Symbol {
        env.storage().instance().get(&DataKey::ItemName).unwrap()
    }

    pub fn get_end_time(env: Env) -> u64 {
        env.storage().instance().get(&DataKey::EndTime).unwrap()
    }

    pub fn get_min_bid(env: Env) -> i128 {
        env.storage().instance().get(&DataKey::MinBid).unwrap()
    }

    pub fn get_highest_bid(env: Env) -> i128 {
        env.storage().instance().get(&DataKey::HighestBid).unwrap()
    }

    pub fn get_highest_bidder(env: Env) -> Option<Address> {
        env.storage().instance().get(&DataKey::HighestBidder)
    }

    pub fn is_finalized(env: Env) -> bool {
        env.storage().instance().get(&DataKey::Finalized).unwrap_or(false)
    }
}

#[cfg(test)]
mod test;
