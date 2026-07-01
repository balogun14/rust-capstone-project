// Amount to send to the Trader wallet.
pub const SEND_AMOUNT: f64 = 20.0;

// Tolerance when comparing floating-point BTC values.
pub const EPSILON: f64 = 0.0001;

// Block reward in regtest mode.
pub const COINBASE_VALUE: f64 = 50.0;

// Number of blocks to mine so the first coinbase matures (100 + 1).
pub const MATURITY_BLOCKS: u64 = 101;

// Single confirmation block after sending the transaction.
pub const CONFIRM_BLOCKS: u64 = 1;
