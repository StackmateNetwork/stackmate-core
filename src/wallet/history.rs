use std::ffi::CString;
use std::os::raw::c_char;

use serde::{Deserialize, Serialize};

use bdk::blockchain::noop_progress;
use bdk::database::MemoryDatabase;
use bdk::TransactionDetails;
use bdk::Wallet;

use crate::config::WalletConfig;
use crate::e::{ErrorKind, S5Error};

/**
*   "fees": 153,
   "height": 2062130,
   "received": 100000,
   "sent": 0,
   "timestamp": 0,
   "transaction": null,
   "txid"
*/

/// FFI Output
#[derive(Serialize, Deserialize, Debug)]
pub struct Transaction {
  pub timestamp: u64,
  pub height: u32,
  pub verified: bool,
  pub txid: String,
  pub received: u64,
  pub sent: u64,
  pub fee: u64,
}

/// FFI Output
#[derive(Serialize, Deserialize, Debug)]
pub struct WalletHistory {
  history: Vec<Transaction>,
}

impl WalletHistory {
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

impl Transaction {
  pub fn from_txdetail(txdetail: TransactionDetails) -> Self {
    Transaction {
      timestamp: match txdetail.confirmation_time.clone() {
        Some(time) => time.timestamp,
        None => 0,
      },
      height: match txdetail.confirmation_time {
        Some(time) => time.height,
        None => 0,
      },
      verified: txdetail.verified,
      txid: txdetail.txid.to_string(),
      received: txdetail.received,
      sent: txdetail.sent,
      fee: txdetail.fee.unwrap_or(0),
    }
  }
}

pub fn sync_history(config: WalletConfig) -> Result<WalletHistory, S5Error> {
  let wallet = match Wallet::new(
    &config.deposit_desc,
    Some(&config.change_desc),
    config.network,
    MemoryDatabase::default(),
    config.client.unwrap(),
  ) {
    Ok(result) => result,
    Err(e) => {
      println!("{:#?}", e);
      return Err(S5Error::new(ErrorKind::Internal, &e.to_string()));
    }
  };

  match wallet.sync(noop_progress(), None) {
    Ok(_) => (),
    Err(_) => return Err(S5Error::new(ErrorKind::Internal, "Wallet-Sync")),
  };

  match wallet.list_transactions(false) {
    Ok(history) => Ok(WalletHistory {
      history: history
        .iter()
        .map(|txdetail| Transaction::from_txdetail(txdetail.clone()))
        .collect(),
    }),
    Err(e) => Err(S5Error::new(ErrorKind::Internal, &e.to_string())),
  }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct WalletBalance {
  pub balance: u64,
}
impl WalletBalance {
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

pub fn sync_balance(config: WalletConfig) -> Result<WalletBalance, S5Error> {
  let wallet = match Wallet::new(
    &config.deposit_desc,
    Some(&config.change_desc),
    config.network,
    MemoryDatabase::default(),
    config.client.unwrap(),
  ) {
    Ok(result) => result,
    Err(e) => {
      println!("{:#?}", e);
      return Err(S5Error::new(ErrorKind::Internal, &e.to_string()));
    }
  };

  match wallet.sync(noop_progress(), None) {
    Ok(_) => (),
    Err(_) => return Err(S5Error::new(ErrorKind::Internal, "Wallet-Sync")),
  };

  match wallet.get_balance() {
    Ok(balance) => Ok(WalletBalance { balance }),
    Err(e) => Err(S5Error::new(ErrorKind::Internal, &e.to_string())),
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::config::{WalletConfig, DEFAULT_TESTNET_NODE};

  #[test]
  fn test_balance() {
    let xkey = "[db7d25b5/84'/1'/6']tpubDCCh4SuT3pSAQ1qAN86qKEzsLoBeiugoGGQeibmieRUKv8z6fCTTmEXsb9yeueBkUWjGVzJr91bCzeCNShorbBqjZV4WRGjz3CrJsCboXUe";
    let descriptor = format!("wpkh({}/*)", xkey);

    let config = WalletConfig::new(&descriptor, DEFAULT_TESTNET_NODE, None).unwrap();
    let balance = sync_balance(config).unwrap();
    assert_eq!((balance.balance>=0), true)
  }
  #[test]
  fn test_history() {
    //   let xkey = "[db7d25b5/84'/1'/6']tpubDCCh4SuT3pSAQ1qAN86qKEzsLoBeiugoGGQeibmieRUKv8z6fCTTmEXsb9yeueBkUWjGVzJr91bCzeCNShorbBqjZV4WRGjz3CrJsCboXUe";
    //   let descriptor = format!("wpkh({}/0/*)", xkey);
    let descriptor = "wpkh([66a0c105/84h/1h/5h]tpubDCKvnVh6U56wTSUEJGamQzdb3ByAc6gTPbjxXQqts5Bf1dBMopknipUUSmAV3UuihKPTddruSZCiqhyiYyhFWhz62SAGuC3PYmtAafUuG6R/0/*)";
    let config = WalletConfig::new(&descriptor, DEFAULT_TESTNET_NODE, None).unwrap();
    let history = sync_history(config).unwrap();
    println!("{:#?}", history);
  }
}
