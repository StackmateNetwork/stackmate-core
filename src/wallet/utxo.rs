use std::ffi::CString;
use std::str;

use std::os::raw::c_char;
use serde::{Deserialize, Serialize};
use bdk::database::MemoryDatabase;
use bdk::{Wallet,SyncOptions};
use bdk::LocalUtxo;
use bitcoin::util::address::Address;
use bitcoin::network::constants::Network;

use crate::config::WalletConfig;
use crate::e::{ErrorKind, S5Error};


/// FFI Output
#[derive(Serialize, Deserialize, Debug)]
pub struct WalletUtxo {
  pub txid: String,
  pub vout: u32,
  pub value: u64,
  pub script_pubkey: String,
  pub keychain_kind: String
}

impl WalletUtxo {
  pub fn from_local_utxo(utxo: LocalUtxo, network: Network) -> Self {
    let address = Address::from_script(&utxo.txout.script_pubkey, network).unwrap();
    WalletUtxo {
      txid: utxo.outpoint.txid.to_string(),
      vout: utxo.outpoint.vout,
      value: utxo.txout.value,
      script_pubkey: address.to_string(),
      keychain_kind: utxo.keychain.as_byte().to_string()
    }
  }
}
/// FFI Output
#[derive(Serialize, Deserialize, Debug)]
pub struct WalletUtxos {
  pub utxos: Vec<WalletUtxo>,
}
impl WalletUtxos {
  pub fn c_stringify(&self) -> *mut c_char {
    let stringified = match serde_json::to_string(self) {
      Ok(result) => result,
      Err(_) => {
        return CString::new("Error:JSON Stringify Failed. BAD NEWS! Contact Support.")
          .unwrap()
          .into_raw()
      }
    };

    CString::new(stringified).unwrap().into_raw()
  }
}

pub fn list_unspent(config: WalletConfig) -> Result<WalletUtxos, S5Error> {
  let network = config.network.clone();
  let wallet = match Wallet::new(
    &config.deposit_desc,
    Some(&config.change_desc),
    config.network,
    MemoryDatabase::default(),
  ) {
    Ok(result) => result,
    Err(e) => {
      println!("{:#?}", e);
      return Err(S5Error::new(ErrorKind::Internal, &e.to_string()));
    }
  };

  match wallet.sync(&config.client.unwrap(), SyncOptions::default()) {
    Ok(_) => (),
    Err(_) => return Err(S5Error::new(ErrorKind::Internal, "Wallet-Sync")),
  };

  match wallet.list_unspent() {
    Ok(result) => Ok(WalletUtxos{
        utxos:result
        .iter()
        .map(|utxo| WalletUtxo::from_local_utxo(utxo.clone(), network))
        .collect(),
    }),
    Err(e) => Err(S5Error::new(ErrorKind::Internal, &e.to_string())),
  }
}



#[cfg(test)]
mod tests {
  use super::*;
  use crate::config::{WalletConfig, DEFAULT_TESTNET_NODE};

  #[test]
  fn test_utxo() {
    let descriptor = "wpkh([8c0a6143/84h/1h/0h]tpubDDjEawrHcboLRccFyt3hcebjhUBPbkueturmkp2EZv3gtaLQLWFeyPVBXVMYt2F5vZcmrwEihVb9axivcQ5QHNnsWgLhmrZyVmq7gHnS4no/*)";
    let config = WalletConfig::new(&descriptor, DEFAULT_TESTNET_NODE, None,None).unwrap();
    let utxos = list_unspent(config).unwrap();
    println!("{:#?}", utxos);
  }
}
