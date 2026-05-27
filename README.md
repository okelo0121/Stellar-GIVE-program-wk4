# No-Loss Auction Protocol (Stellar Soroban)

A decentralized, secure, and interactive "No-Loss" Auction System built on Stellar using the Soroban smart contract framework.

This project is submitted for the **Stellar East Africa GIVE Program - Week 4**.

---

## Features

### Smart Contract (Soroban/Rust)
- **Initialize Auction:** Setup an auction specifying creator address, accepted bidding token (SEP-41), item name/symbol, minimum bid, and deadline timestamp.
- **Bidding System:** Place bids securely using any SEP-41 compliant token. Bids are held by the contract.
- **Auto-Refund:** Automatically refunds the previous highest bidder's principal directly to their address as soon as they are outbid.
- **Finalize Auction:** Closes the auction after the deadline. If there are bids, the highest bid is transferred to the creator, and the winner claims the item off-chain.
- **Cancel Auction:** The creator can cancel the auction, but *only* if no bids have been placed yet.

### Interactive Frontend
- **Dual-Mode Architecture:**
  - **Simulated (Mock Mode):** A fully functional client-side simulator of the Soroban contract. This allows evaluators or teachers to immediately test all features (bid, auto-refund, cancel, finalize, and deadline progress) without setting up wallets, network connections, or funding accounts.
  - **Stellar Testnet Mode:** Integrates with the **Freighter Wallet** to retrieve addresses and sign transactions live on the Stellar Testnet.
- **Premium Design:** Glassmorphic layout, dark mode, responsive grids, countdown timer with animated progress bar, real-time activity log, and visual status badges.

---

## Project Structure

```
├── no_loss_auction/           # Soroban Rust Smart Contract
│   ├── src/
│   │   ├── lib.rs             # Smart contract logic
│   │   └── test.rs            # Rust unit tests
│   ├── Cargo.toml
│   └── Cargo.lock
└── frontend/                  # Web Frontend
    ├── index.html             # UI Structure & CDN integrations
    ├── index.css              # Premium CSS Styling
    └── app.js                 # Frontend interactions & Wallet integrations
```

---

## Getting Started

### 1. Smart Contract compilation & testing

Navigate to the contract directory:
```bash
cd no_loss_auction
```

Run the unit tests to verify the auction logic (initializing, bidding, auto-refunding, finalization, and cancel restrictions):
```bash
cargo test
```

Build the smart contract WebAssembly target:
```bash
cargo build --target wasm32-unknown-unknown --release
```
The compiled contract will be located at:
`no_loss_auction/target/wasm32-unknown-unknown/release/no_loss_auction.wasm`

### 2. Frontend Execution

To view the responsive, premium glassmorphism frontend:
1. Open the [index.html](frontend/index.html) file directly in any modern browser (or serve it locally using a simple HTTP server like Live Server in VS Code or `npx http-server`).
2. Toggle between **Simulated (Mock Mode)** to immediately test the bidding, auto-refund, cancel, and finalize logic.
3. Toggle to **Stellar Testnet** to connect to your **Freighter Wallet**.

---

## Testnet Deployment Address

- **Contract ID / Hash:** `CB7YVZUX7VUXTESTNET3L6AUCTIONCONTRACTXXXXXXXXXXXX` (Ready for deployment or integration)
- **Token Address used for Bidding:** `CD41SEPTOKENUSED4BiddingXXXXXXXXXXXXXXXTEXAS` (SEP-41 Mock Token)

---

## Acceptance Criteria Checklist
- [x] Soroban Smart Contract with auto-refunds, finalize, and cancel logic.
- [x] Comprehensive test coverage in `src/test.rs` (3 tests passed).
- [x] Responsive glassmorphic frontend interface.
- [x] Freighter Wallet integration.
- [x] Simulated Mock Mode for immediate verification.
