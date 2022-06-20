#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use super::*;
    use crate::config::{WalletConfig, DEFAULT_TESTNET_NODE};
    use crate::e::S5Error;
    use crate::key::{derivation, ec, seed};
    use bdk::bitcoin::Txid;
    use bdk::blockchain::GetTx;

    #[test]
    fn test_proof_of_payment() {
        // given a master key, txid and a message
        // create a proof that the payment was made by signing a custom message
        let txid_str = "69ec8f72a3e601e807adb5d778ad0ad27cf5f14dcab59f5fbadf3754442cdcfd";
        let txid = Txid::from_str(txid_str).unwrap();

        let message = "This transaction was made by Vishal Menon to epik.com for the aquisition of the sats.at domain";
        let config = WalletConfig::new("*", DEFAULT_TESTNET_NODE, None).unwrap();
        let transaction = config.client.unwrap().get_tx(&txid).unwrap();
        print!("{:#?}", transaction);
    }
}
// hexdump -C -s 8 -n 285 blk00000.dat
