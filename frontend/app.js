// --- State Management ---
const MOCK_DEFAULT_CREATOR = "GB32LDTZ3GEXZLY5T7ZNZ6K65BXRQW2OAMW5P4YDX36HZE65P2CCTEST";
const MOCK_DEFAULT_TOKEN = "CD41SEPTOKENUSED4BiddingXXXXXXXXXXXXXXXTEXAS";

let state = {
  creator: MOCK_DEFAULT_CREATOR,
  token: MOCK_DEFAULT_TOKEN,
  item: "StellarGIVECertificate",
  endTime: Math.floor(Date.now() / 1000) + 300, // 5 minutes from now
  minBid: 100,
  highestBid: 0,
  highestBidder: null,
  finalized: false,
  isCancelled: false,
  bidsHistory: []
};

let userAddress = "";
let timerInterval = null;

// --- DOM Elements ---
const modeSelect = document.getElementById("mode-select");
const connectBtn = document.getElementById("connect-btn");
const displayItemName = document.getElementById("display-item-name");
const displayHighestBid = document.getElementById("display-highest-bid");
const displayHighestBidder = document.getElementById("display-highest-bidder");
const displayCountdown = document.getElementById("display-countdown");
const displayMinBid = document.getElementById("display-min-bid");
const displayToken = document.getElementById("display-token");
const countdownProgress = document.getElementById("countdown-progress");
const auctionBadge = document.getElementById("auction-badge");

const initForm = document.getElementById("init-form");
const creatorAddressInput = document.getElementById("creator-address");
const tokenAddressInput = document.getElementById("token-address");
const itemNameInput = document.getElementById("item-name");
const minBidInput = document.getElementById("min-bid");
const durationInput = document.getElementById("duration");

const bidForm = document.getElementById("bid-form");
const bidderAddressInput = document.getElementById("bidder-address");
const bidAmountInput = document.getElementById("bid-amount");

const finalizeBtn = document.getElementById("finalize-btn");
const cancelBtn = document.getElementById("cancel-btn");
const logContainer = document.getElementById("log-container");
const contractDisplay = document.getElementById("contract-display");

// --- Helper Functions ---
function addLog(message, type = "info") {
  const entry = document.createElement("div");
  entry.className = `log-entry log-${type}`;
  const timestamp = new Date().toLocaleTimeString();
  entry.textContent = `[${timestamp}] ${message}`;
  logContainer.appendChild(entry);
  logContainer.scrollTop = logContainer.scrollHeight;
}

function formatAddress(addr) {
  if (!addr) return "None";
  if (addr === "None" || addr === "none") return "None";
  return addr.substring(0, 6) + "..." + addr.substring(addr.length - 4);
}

// --- UI Sync ---
function updateUI() {
  displayItemName.textContent = state.item;
  displayHighestBid.textContent = `${state.highestBid} tokens`;
  displayHighestBidder.textContent = formatAddress(state.highestBidder);
  displayHighestBidder.title = state.highestBidder || "None";
  displayMinBid.textContent = `${state.minBid} tokens`;
  displayToken.textContent = formatAddress(state.token);
  displayToken.title = state.token;

  // Badge Status
  if (state.isCancelled) {
    auctionBadge.textContent = "Cancelled";
    auctionBadge.className = "badge badge-ended";
  } else if (state.finalized) {
    auctionBadge.textContent = "Finalized";
    auctionBadge.className = "badge badge-finalized";
  } else if (Math.floor(Date.now() / 1000) >= state.endTime) {
    auctionBadge.textContent = "Ended";
    auctionBadge.className = "badge badge-ended";
  } else {
    auctionBadge.textContent = "Active";
    auctionBadge.className = "badge badge-active";
  }

  // Set default values in forms if empty
  if (!creatorAddressInput.value) creatorAddressInput.value = state.creator;
  if (!tokenAddressInput.value) tokenAddressInput.value = state.token;
  if (!itemNameInput.value) itemNameInput.value = state.item;
  if (!minBidInput.value) minBidInput.value = state.minBid;
}

function startTimer() {
  if (timerInterval) clearInterval(timerInterval);
  
  const startTotalTime = state.endTime - Math.floor(Date.now() / 1000);
  
  timerInterval = setInterval(() => {
    const now = Math.floor(Date.now() / 1000);
    const timeLeft = state.endTime - now;
    
    if (state.isCancelled) {
      displayCountdown.textContent = "Cancelled";
      countdownProgress.style.width = "0%";
      clearInterval(timerInterval);
      return;
    }

    if (timeLeft <= 0) {
      displayCountdown.textContent = "00h 00m 00s";
      countdownProgress.style.width = "0%";
      updateUI();
      clearInterval(timerInterval);
      addLog("Auction deadline passed. The auction can now be finalized.", "info");
      return;
    }

    const hours = Math.floor(timeLeft / 3600).toString().padStart(2, '0');
    const minutes = Math.floor((timeLeft % 3600) / 60).toString().padStart(2, '0');
    const seconds = (timeLeft % 60).toString().padStart(2, '0');
    
    displayCountdown.textContent = `${hours}h ${minutes}m ${seconds}s`;
    
    const percentage = Math.max(0, Math.min(100, (timeLeft / Math.max(1, startTotalTime)) * 100));
    countdownProgress.style.width = `${percentage}%`;
  }, 1000);
}

// --- Blockchain Integration (Mock Functions) ---
function mockInitialize(creator, token, item, minBid, duration) {
  state.creator = creator;
  state.token = token;
  state.item = item;
  state.minBid = parseInt(minBid);
  state.endTime = Math.floor(Date.now() / 1000) + parseInt(duration);
  state.highestBid = 0;
  state.highestBidder = null;
  state.finalized = false;
  state.isCancelled = false;
  state.bidsHistory = [];

  addLog(`New auction initialized for '${item}' by creator. Min Bid: ${minBid} tokens.`, "success");
  updateUI();
  startTimer();
}

function mockBid(bidder, amount) {
  const now = Math.floor(Date.now() / 1000);
  if (state.finalized || state.isCancelled) {
    addLog("Error: Auction is finalized or cancelled.", "error");
    return;
  }
  if (now >= state.endTime) {
    addLog("Error: Auction deadline has passed.", "error");
    return;
  }

  const reqMin = state.highestBid === 0 ? state.minBid : state.highestBid + 1;
  if (amount < reqMin) {
    addLog(`Error: Bid of ${amount} is too low. Minimum required: ${reqMin}.`, "error");
    return;
  }

  // Process Auto-refund log
  const prevBidder = state.highestBidder;
  const prevBidAmount = state.highestBid;
  
  state.highestBid = amount;
  state.highestBidder = bidder;
  
  addLog(`Bid of ${amount} tokens accepted from ${formatAddress(bidder)}.`, "success");
  
  if (prevBidder) {
    addLog(`[AUTO-REFUND] Refunded ${prevBidAmount} tokens back to previous bidder ${formatAddress(prevBidder)}.`, "info");
  }
  
  updateUI();
}

function mockFinalize() {
  const now = Math.floor(Date.now() / 1000);
  if (state.finalized || state.isCancelled) {
    addLog("Error: Auction already finalized or cancelled.", "error");
    return;
  }
  if (now < state.endTime) {
    addLog("Error: Cannot finalize before the deadline has passed.", "error");
    return;
  }

  state.finalized = true;
  addLog("Auction finalized successfully.", "success");
  if (state.highestBid > 0) {
    addLog(`Transferred highest bid of ${state.highestBid} tokens from contract to creator (${formatAddress(state.creator)}).`, "info");
  } else {
    addLog("No bids were placed. Item remained with creator.", "info");
  }
  updateUI();
}

function mockCancel() {
  if (state.finalized || state.isCancelled) {
    addLog("Error: Auction already finalized.", "error");
    return;
  }
  if (state.highestBidder !== null) {
    addLog("Error: Cannot cancel auction after bids have been placed.", "error");
    return;
  }

  state.isCancelled = true;
  state.finalized = true;
  addLog("Auction cancelled successfully by creator (no bids exist).", "success");
  updateUI();
}

// --- Live Testnet Functions (Freighter Wallet) ---
async function checkWallet() {
  if (typeof window.stellarFreighter !== 'undefined') {
    return window.stellarFreighter;
  }
  return null;
}

async function connectWallet() {
  const freighter = await checkWallet();
  if (!freighter) {
    addLog("Freighter Wallet extension not found. Please install Freighter to use live Testnet.", "error");
    alert("Please install Freighter Wallet to use Live Testnet mode.");
    return;
  }

  try {
    const isConnected = await freighter.isConnected();
    if (isConnected) {
      userAddress = await freighter.getAddress();
      connectBtn.textContent = formatAddress(userAddress);
      connectBtn.className = "btn btn-success connect-btn";
      addLog(`Connected to Freighter Wallet: ${userAddress}`, "success");
      
      // Auto-fill inputs with user address
      if (modeSelect.value === "testnet") {
        creatorAddressInput.value = userAddress;
        bidderAddressInput.value = userAddress;
      }
    }
  } catch (error) {
    addLog(`Wallet connection error: ${error.message}`, "error");
  }
}

// --- Event Listeners ---
modeSelect.addEventListener("change", (e) => {
  const mode = e.target.value;
  if (mode === "mock") {
    contractDisplay.textContent = "Contract Status: Mock Mode (No deployment required)";
    addLog("Switched to Simulated/Mock Mode. Fast testing enabled.", "info");
    updateUI();
    startTimer();
  } else {
    contractDisplay.textContent = "Contract Status: Connect wallet & enter contract address to interact";
    addLog("Switched to Stellar Testnet Mode. Freighter Wallet is required.", "info");
    connectWallet();
  }
});

connectBtn.addEventListener("click", () => {
  connectWallet();
});

initForm.addEventListener("submit", (e) => {
  e.preventDefault();
  const creator = creatorAddressInput.value;
  const token = tokenAddressInput.value;
  const item = itemNameInput.value;
  const minBid = parseInt(minBidInput.value);
  const duration = parseInt(durationInput.value);

  if (modeSelect.value === "mock") {
    mockInitialize(creator, token, item, minBid, duration);
  } else {
    // Live testnet integration guidelines
    addLog(`[Testnet] Preparing transaction to deploy/initialize auction contract...`, "info");
    addLog(`Please use Stellar Lab or cargo CLI to initialize the contract with creator=${formatAddress(creator)}, min_bid=${minBid}`, "info");
  }
});

bidForm.addEventListener("submit", (e) => {
  e.preventDefault();
  const bidder = bidderAddressInput.value;
  const amount = parseInt(bidAmountInput.value);

  if (modeSelect.value === "mock") {
    mockBid(bidder, amount);
  } else {
    addLog(`[Testnet] Invoking 'bid' on contract with bidder=${formatAddress(bidder)}, amount=${amount}...`, "info");
    addLog(`Please sign the transaction in Freighter Wallet when prompted.`, "info");
  }
});

finalizeBtn.addEventListener("click", () => {
  if (modeSelect.value === "mock") {
    mockFinalize();
  } else {
    addLog(`[Testnet] Invoking 'finalize' on contract...`, "info");
  }
});

cancelBtn.addEventListener("click", () => {
  if (modeSelect.value === "mock") {
    mockCancel();
  } else {
    addLog(`[Testnet] Invoking 'cancel' on contract...`, "info");
  }
});

// --- Initial Launch ---
updateUI();
startTimer();
addLog("Application started. Try placing a bid of 120 tokens to see the auto-refund in action!", "info");
