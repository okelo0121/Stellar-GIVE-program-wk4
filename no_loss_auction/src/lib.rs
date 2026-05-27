#![no_std]
use soroban_sdk::{contract, contracterror, contractimpl, contracttype, token, Address, Env, Symbol};

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Error {
    AlreadyInitialized = 1,
    InvalidEndTime = 2,
    InvalidMinBid = 3,
    AuctionEnded = 4,
    AuctionFinalized = 5,
    BidTooLow = 6,
    DeadlineNotPassed = 7,
    BidsExist = 8,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AuctionState {
    pub creator: Address,
    pub token: Address,
    pub item: Symbol,
    pub end_time: u64,
    pub min_bid: i128,
    pub highest_bid: i128,
    pub highest_bidder: Option<Address>,
    pub finalized: bool,
}

#[contracttype]
pub enum DataKey {
    State,
}

#[contract]
pub struct NoLossAuctionContract;

#[contractimpl]
impl NoLossAuctionContract {
    pub fn initialize(
        env: Env,
        creator: Address,
        token: Address,
        item: Symbol,
        end_time: u64,
        min_bid: i128,
    ) -> Result<(), Error> {
        if env.storage().instance().has(&DataKey::State) {
            return Err(Error::AlreadyInitialized);
        }
        if end_time <= env.ledger().timestamp() {
            return Err(Error::InvalidEndTime);
        }
        if min_bid <= 0 {
            return Err(Error::InvalidMinBid);
        }

        let state = AuctionState {
            creator,
            token,
            item,
            end_time,
            min_bid,
            highest_bid: 0,
            highest_bidder: None,
            finalized: false,
        };

        env.storage().instance().set(&DataKey::State, &state);
        Ok(())
    }

    pub fn bid(env: Env, bidder: Address, amount: i128) -> Result<(), Error> {
        bidder.require_auth();

        let mut state: AuctionState = env.storage().instance().get(&DataKey::State)
            .ok_or(Error::AlreadyInitialized)?;

        if state.finalized {
            return Err(Error::AuctionFinalized);
        }
        if env.ledger().timestamp() >= state.end_time {
            return Err(Error::AuctionEnded);
        }

        let required_min = if state.highest_bid == 0 { state.min_bid } else { state.highest_bid + 1 };
        if amount < required_min {
            return Err(Error::BidTooLow);
        }

        let token_client = token::Client::new(&env, &state.token);

        if let Some(prev_bidder) = state.highest_bidder {
            token_client.transfer(&env.current_contract_address(), &prev_bidder, &state.highest_bid);
        }

        token_client.transfer(&bidder, &env.current_contract_address(), &amount);

        state.highest_bidder = Some(bidder);
        state.highest_bid = amount;

        env.storage().instance().set(&DataKey::State, &state);
        Ok(())
    }

    pub fn finalize(env: Env) -> Result<(), Error> {
        let mut state: AuctionState = env.storage().instance().get(&DataKey::State)
            .ok_or(Error::AlreadyInitialized)?;

        if state.finalized {
            return Err(Error::AuctionFinalized);
        }
        if env.ledger().timestamp() < state.end_time {
            return Err(Error::DeadlineNotPassed);
        }

        if state.highest_bid > 0 {
            let token_client = token::Client::new(&env, &state.token);
            token_client.transfer(&env.current_contract_address(), &state.creator, &state.highest_bid);
        }

        state.finalized = true;
        env.storage().instance().set(&DataKey::State, &state);
        Ok(())
    }

    pub fn cancel(env: Env) -> Result<(), Error> {
        let mut state: AuctionState = env.storage().instance().get(&DataKey::State)
            .ok_or(Error::AlreadyInitialized)?;

        state.creator.require_auth();

        if state.finalized {
            return Err(Error::AuctionFinalized);
        }
        if state.highest_bidder.is_some() {
            return Err(Error::BidsExist);
        }

        state.finalized = true;
        env.storage().instance().set(&DataKey::State, &state);
        Ok(())
    }

    pub fn get_state(env: Env) -> Option<AuctionState> {
        env.storage().instance().get(&DataKey::State)
    }
}
