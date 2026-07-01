use bitcoincore_rpc::{Auth, Client, RpcApi};
use serde_json::{json, Value};

pub const RPC_URL: &str = "http://127.0.0.1:18443";
pub const RPC_USER: &str = "alice";
pub const RPC_PASS: &str = "password";

pub fn connect(wallet: Option<&str>) -> bitcoincore_rpc::Result<Client> {
    let url = match wallet {
        Some(w) => format!("{}/wallet/{}", RPC_URL, w),
        None => RPC_URL.to_string(),
    };
    Client::new(
        &url,
        Auth::UserPass(RPC_USER.to_owned(), RPC_PASS.to_owned()),
    )
}

pub fn ensure_wallet(rpc: &Client, name: &str) -> bitcoincore_rpc::Result<()> {
    let wallets: Vec<String> = rpc.call("listwallets", &[] as &[Value])?;
    if wallets.contains(&name.to_string()) {
        return Ok(());
    }
    if rpc.call::<Value>("loadwallet", &[json!(name)]).is_err() {
        rpc.call::<Value>("createwallet", &[json!(name)])?;
    }
    Ok(())
}
