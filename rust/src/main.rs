use bitcoincore_rpc::{Client, RpcApi};
use serde::Deserialize;
use serde_json::json;

mod client;
mod config;
mod extract;
mod output;
mod transaction;

// Node access params
#[allow(dead_code)]
const RPC_URL: &str = "http://127.0.0.1:18443"; // Default regtest RPC port
#[allow(dead_code)]
const RPC_USER: &str = "alice";
#[allow(dead_code)]
const RPC_PASS: &str = "password";

// You can use calls not provided in RPC lib API using the generic `call` function.
// An example of using the `send` RPC call, which doesn't have exposed API.
// You can also use serde_json `Deserialize` derivation to capture the returned json result.
#[allow(dead_code)]
fn send(rpc: &Client, addr: &str) -> bitcoincore_rpc::Result<String> {
    let args = [
        json!([{addr : 100 }]), // recipient address
        json!(null),            // conf target
        json!(null),            // estimate mode
        json!(null),            // fee rate in sats/vb
        json!(null),            // Empty option object
    ];

    #[derive(Deserialize)]
    struct SendResult {
        complete: bool,
        txid: String,
    }
    let send_result = rpc.call::<SendResult>("send", &args)?;
    assert!(send_result.complete);
    Ok(send_result.txid)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Connect to Bitcoin Core RPC
    let rpc = client::connect(None)?;

    // Get blockchain info
    let blockchain_info = rpc.get_blockchain_info()?;
    println!("Blockchain Info: {:?}", blockchain_info);

    // Create/Load the wallets, named 'Miner' and 'Trader'. Have logic to optionally create/load them if they do not exist or not loaded already.
    client::ensure_wallet(&rpc, "Miner")?;
    client::ensure_wallet(&rpc, "Trader")?;

    // Generate spendable balances in the Miner wallet. How many blocks needs to be mined?
    let miner = client::connect(Some("Miner"))?;
    let miner_addr = miner
        .get_new_address(None, None)?
        .assume_checked()
        .to_string();
    transaction::mine_blocks(&rpc, config::MATURITY_BLOCKS, &miner_addr)?;

    // Load Trader wallet and generate a new address
    let trader = client::connect(Some("Trader"))?;
    let trader_addr = trader
        .get_new_address(None, None)?
        .assume_checked()
        .to_string();

    // Send 20 BTC from Miner to Trader
    let txid = transaction::send_to(&miner, &trader_addr, config::SEND_AMOUNT)?;

    // Check transaction in mempool
    let mempool = miner.get_raw_mempool()?;
    if !mempool.contains(&txid) {
        return Err("transaction not found in mempool".into());
    }

    // Mine 1 block to confirm the transaction
    transaction::mine_blocks(&rpc, config::CONFIRM_BLOCKS, &miner_addr)?;

    // Extract all required transaction details
    let tx_info: extract::TransactionInfo = miner.call(
        "gettransaction",
        &[json!(txid.to_string()), json!(null), json!(true)],
    )?;

    let extracted = extract::parse(tx_info, &txid.to_string(), &trader_addr, &miner_addr)?;

    // Write the data to ../out.txt in the specified format given in readme.md
    output::write("../out.txt", &extracted)?;

    Ok(())
}
