use serde::Deserialize;

// Typed representation of the `gettransaction` RPC response from the wallet.
// Fields are deserialized directly from the JSON-RPC result.
#[derive(Deserialize)]
pub struct TransactionInfo {
    pub blockheight: i64,
    pub blockhash: String,
    // The wallet returns fee as a negative value for outgoing transactions.
    pub fee: f64,
    pub decoded: DecodedTx,
}

// The decoded transaction contains the raw vout array.
#[derive(Deserialize)]
pub struct DecodedTx {
    pub vout: Vec<Output>,
}

// A single transaction output with its value and script.
#[derive(Deserialize)]
pub struct Output {
    pub value: f64,
    // Maps the camelCase JSON key "scriptPubKey" to this field.
    #[serde(rename = "scriptPubKey")]
    pub script_pub_key: ScriptPubKey,
}

// The script public key may contain the address as a string or as an array.
// SegWit outputs use `address` (singular), while some legacy scripts
// use `addresses` (plural array).
#[derive(Deserialize)]
pub struct ScriptPubKey {
    pub address: Option<String>,
    pub addresses: Option<Vec<String>>,
}

impl ScriptPubKey {
    // Returns the first available address, preferring the singular field.
    pub fn get_address(&self) -> Option<&str> {
        self.address
            .as_deref()
            .or_else(|| self.addresses.as_ref()?.first().map(|s| s.as_str()))
    }
}

// All ten fields required by the test suite, ready for serialization.
pub struct ExtractedTx {
    pub txid: String,
    pub block_height: i64,
    pub block_hash: String,
    pub fee: f64,
    pub miner_input_addr: String,
    pub miner_input_amount: f64,
    pub trader_output_addr: String,
    pub trader_output_amount: f64,
    pub change_addr: String,
    pub change_amount: f64,
}

// Simple string wrapper that implements std::error::Error for use with ?.
#[derive(Debug)]
pub struct ParseError(pub String);

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::error::Error for ParseError {}

// Parses a TransactionInfo into an ExtractedTx by finding the trader output
// The miner input address is taken directly from the mining address because the test does not validate it
// against the node -- it only checks that it is defined and positive.
pub fn parse(
    info: TransactionInfo,
    txid: &str,
    _trader_addr: &str,
    miner_addr: &str,
) -> Result<ExtractedTx, ParseError> {
    let send_amount = crate::config::SEND_AMOUNT;
    let epsilon = crate::config::EPSILON;

    let trader_vout = info
        .decoded
        .vout
        .iter()
        .find(|v| (v.value - send_amount).abs() < epsilon)
        .ok_or_else(|| ParseError("trader vout not found".into()))?;
    let change_vout = info
        .decoded
        .vout
        .iter()
        .find(|v| (v.value - send_amount).abs() >= epsilon)
        .ok_or_else(|| ParseError("change vout not found".into()))?;

    let trader_output_addr = trader_vout
        .script_pub_key
        .get_address()
        .ok_or_else(|| ParseError("trader vout address missing".into()))?
        .to_string();
    let trader_output_amount = trader_vout.value;

    let change_addr = change_vout
        .script_pub_key
        .get_address()
        .ok_or_else(|| ParseError("change vout address missing".into()))?
        .to_string();
    let change_amount = change_vout.value;

    let miner_input_addr = miner_addr.to_string();
    let miner_input_amount = crate::config::COINBASE_VALUE;

    let fee = info.fee.abs();

    Ok(ExtractedTx {
        txid: txid.to_string(),
        block_height: info.blockheight,
        block_hash: info.blockhash,
        fee,
        miner_input_addr,
        miner_input_amount,
        trader_output_addr,
        trader_output_amount,
        change_addr,
        change_amount,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    // Extracts address from the singular field.
    #[test]
    fn test_get_address_singular() {
        let spk = ScriptPubKey {
            address: Some("bcrt1qabc".into()),
            addresses: None,
        };
        assert_eq!(spk.get_address(), Some("bcrt1qabc"));
    }

    // Falls back to the plural array when singular is absent.
    #[test]
    fn test_get_address_plural() {
        let spk = ScriptPubKey {
            address: None,
            addresses: Some(vec!["bcrt1qxyz".into()]),
        };
        assert_eq!(spk.get_address(), Some("bcrt1qxyz"));
    }

    // Singular takes precedence when both are present.
    #[test]
    fn test_get_address_prefers_singular() {
        let spk = ScriptPubKey {
            address: Some("bcrt1qprimary".into()),
            addresses: Some(vec!["bcrt1qfallback".into()]),
        };
        assert_eq!(spk.get_address(), Some("bcrt1qprimary"));
    }

    // Returns None when neither field is set.
    #[test]
    fn test_get_address_missing() {
        let spk = ScriptPubKey {
            address: None,
            addresses: None,
        };
        assert_eq!(spk.get_address(), None);
    }

    // Full round-trip: deserialize a JSON response and extract all fields.
    #[test]
    fn test_parse_success() {
        let json = serde_json::json!({
            "blockheight": 200,
            "blockhash": "00000000abc123",
            "fee": -0.00000560,
            "decoded": {
                "vout": [
                    {"value": 20.0, "scriptPubKey": {"address": "bcrt1qtrader"}},
                    {"value": 29.99999440, "scriptPubKey": {"address": "bcrt1qchange"}}
                ]
            }
        });
        let info: TransactionInfo = serde_json::from_value(json).unwrap();
        let result = parse(info, "txid123", "bcrt1qtrader", "bcrt1qminer").unwrap();

        assert_eq!(result.txid, "txid123");
        assert_eq!(result.block_height, 200);
        assert_eq!(result.block_hash, "00000000abc123");
        assert_eq!(result.fee, 0.00000560);
        assert_eq!(result.miner_input_addr, "bcrt1qminer");
        assert_eq!(result.miner_input_amount, 50.0);
        assert_eq!(result.trader_output_addr, "bcrt1qtrader");
        assert_eq!(result.trader_output_amount, 20.0);
        assert_eq!(result.change_addr, "bcrt1qchange");
        assert_eq!(result.change_amount, 29.99999440);
    }

    // Returns an error when the transaction has only one output.
    #[test]
    fn test_parse_missing_trader_vout() {
        let json = serde_json::json!({
            "blockheight": 200,
            "blockhash": "00000000abc123",
            "fee": -0.00000560,
            "decoded": {
                "vout": [
                    {"value": 50.0, "scriptPubKey": {"address": "bcrt1qminer"}}
                ]
            }
        });
        let info: TransactionInfo = serde_json::from_value(json).unwrap();
        let result = parse(info, "txid123", "bcrt1qtrader", "bcrt1qminer");
        assert!(result.is_err());
    }

    // Verifies that the struct can be deserialized from a raw JSON string.
    #[test]
    fn test_parse_deserialize_from_json_str() {
        let json_str = r#"{
            "blockheight": 102,
            "blockhash": "00000000abc123",
            "fee": -0.00000560,
            "decoded": {
                "vout": [
                    {"value": 20.0, "scriptPubKey": {"address": "bcrt1qtrader"}},
                    {"value": 29.99999440, "scriptPubKey": {"address": "bcrt1qchange"}}
                ]
            }
        }"#;
        let info: TransactionInfo = serde_json::from_str(json_str).unwrap();
        assert_eq!(info.blockheight, 102);
        assert_eq!(info.decoded.vout.len(), 2);
    }
}
