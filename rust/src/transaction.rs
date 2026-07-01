use bitcoincore_rpc::{Client, RpcApi};
use serde::Deserialize;
use serde_json::{json, Value};
use std::str::FromStr;

// Helper that wraps a string message into a boxed error for use with ?.
fn rpc_err(msg: &str) -> Box<dyn std::error::Error> {
    Box::new(std::io::Error::other(msg))
}

// Sends `amount` BTC to `addr` using exactly one UTXO from the wallet.
// Picks a single spendable UTXO via listunspent and passes it as the
// `inputs` option to the `send` RPC. Returns the transaction ID.
pub fn send_to(
    rpc: &Client,
    addr: &str,
    amount: f64,
) -> Result<bitcoincore_rpc::bitcoin::Txid, Box<dyn std::error::Error>> {
    let utxos = rpc.call::<Value>(
        "listunspent",
        &[
            json!(1),
            json!(9999999),
            json!([]),
            json!(true),
            json!({"minimumAmount": amount}),
        ],
    )?;
    let utxos_arr = utxos
        .as_array()
        .ok_or_else(|| rpc_err("listunspent returned non-array"))?;
    let utxo = utxos_arr
        .first()
        .ok_or_else(|| rpc_err("no spendable UTXO with sufficient balance"))?;

    let options = json!({
        "inputs": [{"txid": utxo["txid"], "vout": utxo["vout"]}],
    });

    let args = [
        json!({(addr): amount}),
        json!(null),
        json!(null),
        json!(null),
        options,
    ];

    #[derive(Deserialize)]
    struct SendResult {
        complete: bool,
        txid: String,
    }
    let result = rpc.call::<SendResult>("send", &args)?;
    if !result.complete {
        return Err(rpc_err("send returned incomplete PSBT"));
    }
    let txid = bitcoincore_rpc::bitcoin::Txid::from_str(&result.txid)
        .map_err(|_| rpc_err("invalid txid hex"))?;
    Ok(txid)
}

// Mines `n` blocks to `addr` using the generatetoaddress RPC.
// The `rpc` client must point to the node, not a wallet.
pub fn mine_blocks(rpc: &Client, n: u64, addr: &str) -> Result<Value, Box<dyn std::error::Error>> {
    let args = [json!(n), json!(addr)];
    Ok(rpc.call("generatetoaddress", &args)?)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mine_blocks_args_format() {
        let args = [json!(101u64), json!("bcrt1qtest")];
        assert_eq!(args[0], json!(101));
        assert_eq!(args[1], json!("bcrt1qtest"));
    }
}
