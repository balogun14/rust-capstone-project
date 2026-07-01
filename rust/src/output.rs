use std::fs::File;
use std::io::Write;

use crate::extract::ExtractedTx;

// Writes the ten required fields to a file, one per line.
// Amounts are formatted with 8 decimal places to match the convention
// used by Bitcoin Core in its JSON-RPC responses.
pub fn write(path: &str, tx: &ExtractedTx) -> std::io::Result<()> {
    let mut file = File::create(path)?;
    writeln!(file, "{}", tx.txid)?;
    writeln!(file, "{}", tx.miner_input_addr)?;
    writeln!(file, "{:.8}", tx.miner_input_amount)?;
    writeln!(file, "{}", tx.trader_output_addr)?;
    writeln!(file, "{:.8}", tx.trader_output_amount)?;
    writeln!(file, "{}", tx.change_addr)?;
    writeln!(file, "{:.8}", tx.change_amount)?;
    writeln!(file, "{:.8}", tx.fee)?;
    writeln!(file, "{}", tx.block_height)?;
    writeln!(file, "{}", tx.block_hash)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::extract::ExtractedTx;

    fn sample_tx() -> ExtractedTx {
        ExtractedTx {
            txid: "abc".into(),
            block_height: 100,
            block_hash: "def".into(),
            fee: 0.00000560,
            miner_input_addr: "addr_miner".into(),
            miner_input_amount: 50.0,
            trader_output_addr: "addr_trader".into(),
            trader_output_amount: 20.0,
            change_addr: "addr_change".into(),
            change_amount: 29.99999440,
        }
    }

    // Writes a sample ExtractedTx to a temporary file and verifies that
    // the output matches the exepected 10-line format.
    #[test]
    fn test_write_output_format() {
        let tx = sample_tx();
        let path = "test_out.txt".to_string();

        write(&path, &tx).unwrap();

        let content = std::fs::read_to_string(&path).unwrap();
        let lines: Vec<&str> = content.trim().split('\n').collect();

        assert_eq!(lines.len(), 10);
        assert_eq!(lines[0], "abc");
        assert_eq!(lines[1], "addr_miner");
        assert_eq!(lines[2], "50.00000000");
        assert_eq!(lines[3], "addr_trader");
        assert_eq!(lines[4], "20.00000000");
        assert_eq!(lines[5], "addr_change");
        assert_eq!(lines[6], "29.99999440");
        assert_eq!(lines[7], "0.00000560");
        assert_eq!(lines[8], "100");
        assert_eq!(lines[9], "def");

        std::fs::remove_file(&path).unwrap();
    }
}
