use serde::{Deserialize, Serialize};
use std::ffi::CString;
use std::os::raw::c_char;

use crate::config::{WalletConfig};
use crate::e::{ErrorKind, S5Error};
use bdk::FeeRate;
use bdk::{SignOptions, Wallet};
use bitcoin::util::address::Address;

use bdk::blockchain::noop_progress;
use bdk::database::MemoryDatabase;
use bitcoin::consensus::deserialize;
use bitcoin::network::constants::Network;
use bitcoin::util::psbt::PartiallySignedTransaction;
use bitcoin::blockdata::transaction::Transaction;

use std::str::FromStr;

// use bdk::Error;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct WalletPSBT {
  pub psbt: String,
  pub is_finalized: bool,
}

// impl Clone for WalletPSBT {
//     fn clone(&self) -> WalletPSBT {
//         self
//     }
// }
impl WalletPSBT {
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


pub fn build(
  config: WalletConfig,
  to: &str,
  amount: Option<u64>,
  fee_rate: f32,
  // fee_absolute: Option<u32>,
  sweep: bool
) -> Result<WalletPSBT, S5Error> {
  let wallet = match Wallet::new(
    &config.deposit_desc,
    Some(&config.change_desc),
    config.network,
    MemoryDatabase::default(),
    config.client,
  ) {
    Ok(result) => result,
    Err(_) => return Err(S5Error::new(ErrorKind::Internal, "Wallet-Initialization")),
  };

  match wallet.sync(noop_progress(), None) {
    Ok(_) => (),
    Err(_) => return Err(S5Error::new(ErrorKind::Internal, "Wallet-Sync")),
  };

  let send_to = match Address::from_str(to) {
    Ok(result) => result,
    Err(_) => return Err(S5Error::new(ErrorKind::Internal, "Address-Parse")),
  };

  let (psbt, details) = {
    let mut builder = wallet.build_tx();
    if sweep && amount.is_none(){
      builder
      .drain_wallet()
      .drain_to(send_to.script_pubkey());
    }
    else{
      builder
      .enable_rbf()
      .add_recipient(send_to.script_pubkey(),amount.unwrap());
    }

    builder.fee_rate(FeeRate::from_sat_per_vb(fee_rate));

    match builder.finish() {
      Ok(result) => result,
      Err(e) => {
        println!("{:?}", e);
        return Err(S5Error::new(ErrorKind::Internal, "Transaction-Build"));
      }
    }
    
  };

  println!("Transaction details: {:#?}", details);

  Ok(WalletPSBT {
    psbt: psbt.to_string(),
    is_finalized: false,
  })
}

// pub fn rate_to_absolute(
//   config: WalletConfig,
//   to: &str,
//   amount: u64,
//   fee_absolute: u64,
// ) -> Result<WalletPSBT, S5Error> {
//   let wallet = match Wallet::new(
//     &config.deposit_desc,
//     Some(&config.change_desc),
//     config.network,
//     MemoryDatabase::default(),
//     config.client,
//   ) {
//     Ok(result) => result,
//     Err(_) => return Err(S5Error::new(ErrorKind::Internal, "Wallet-Initialization")),
//   };

//   match wallet.sync(noop_progress(), None) {
//     Ok(_) => (),
//     Err(_) => return Err(S5Error::new(ErrorKind::Internal, "Wallet-Sync")),
//   };

//   let send_to = match Address::from_str(to) {
//     Ok(result) => result,
//     Err(_) => return Err(S5Error::new(ErrorKind::Internal, "Address-Parse")),
//   };

//   let (psbt, details) = {
//     let mut builder = wallet.build_tx();
   
//     builder
//     .enable_rbf()
//     .add_recipient(send_to.script_pubkey(),amount)
//     .fee_absolute(fee_absolute);

//     match builder.finish() {
//       Ok(result) => result,
//       Err(e) => {
//         println!("{:?}", e);
//         return Err(S5Error::new(ErrorKind::Internal, "Transaction-Build"));
//       }
//     }
    
//   };

//   println!("Transaction details: {:#?}", details);

//   Ok(WalletPSBT {
//     psbt: psbt.to_string(),
//     is_finalized: false,
//   })
// }

#[derive(Serialize, Debug, Clone)]
pub struct DecodedTxOutput {
  value: u64,
  to: String,
}

#[derive(Serialize, Debug, Clone)]
pub struct DecodedTx {
  pub outputs: Vec<DecodedTxOutput>,
  pub size: usize,
}

impl DecodedTx {
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

pub fn decode(network: Network, psbt: &str) -> Result<DecodedTx, S5Error> {
  let decoded_psbt = match base64::decode(psbt) {
    Ok(psbt) => psbt,
    Err(_) => return Err(S5Error::new(ErrorKind::Internal, "Basae64-Decode")),
  };

  let psbt_struct: PartiallySignedTransaction = match deserialize(&decoded_psbt) {
    Ok(psbt) => psbt,
    Err(_) => return Err(S5Error::new(ErrorKind::Internal, "Deserialize-Error")),
  };

  println!("{:#?}", &psbt_struct);

  let outputs = &psbt_struct.global.unsigned_tx.output;
  // println!("{:#?}", &outputs);
  // println!("{:#?}", Address::from_script(&outputs[0].clone().script_pubkey,network_enum));
  let inputs = &psbt_struct.inputs;

  let mut decoded_outputs: Vec<DecodedTxOutput> = vec![];
  let mut total_out_value = 0;
  let mut total_in_value = 0;

  let transaction: Transaction = psbt_struct.clone().extract_tx(); 
  let size = transaction.get_size();
  
  for output in outputs {
    total_out_value += output.value;
    decoded_outputs.push(DecodedTxOutput {
      value: output.value,
      to: match Address::from_script(&output.script_pubkey, network) {
        Some(address) => address.to_string(),
        None => "None".to_string(),
      },
    });
  }
  for input in inputs {
    total_in_value += input.witness_utxo.clone().unwrap().value;
  }
  decoded_outputs.push(DecodedTxOutput {
    value: total_in_value - total_out_value,
    to: "miner".to_string(),
  });

  Ok(DecodedTx {
    outputs: decoded_outputs,
    size,
  })
}

pub fn sign(config: WalletConfig, psbt: &str) -> Result<WalletPSBT, S5Error> {
  let wallet = match Wallet::new_offline(
    &config.deposit_desc,
    Some(&config.change_desc),
    config.network,
    MemoryDatabase::default(),
  ) {
    Ok(result) => result,
    Err(_) => return Err(S5Error::new(ErrorKind::Internal, "Wallet-Initialization")),
  };

  let mut final_psbt = match deserialize(&base64::decode(psbt).unwrap()) {
    Ok(psbt) => psbt,
    Err(_) => return Err(S5Error::new(ErrorKind::Internal, "Deserialize-Psbt-Error")),
  };

  let finalized = match wallet.sign(&mut final_psbt, SignOptions::default()) {
    Ok(result) => result,
    Err(_) => return Err(S5Error::new(ErrorKind::Internal, "Sign-Error")),
  };

  Ok(WalletPSBT {
    psbt: final_psbt.to_string(),
    is_finalized: finalized,
  })
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Txid {
  pub txid: String,
}
impl Txid {
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

pub fn broadcast(config: WalletConfig, psbt: &str) -> Result<Txid, S5Error> {
  let wallet = match Wallet::new(
    &config.deposit_desc,
    Some(&config.change_desc),
    config.network,
    MemoryDatabase::default(),
    config.client,
  ) {
    Ok(result) => result,
    Err(_) => return Err(S5Error::new(ErrorKind::Internal, "Wallet-Initialization")),
  };

  match wallet.sync(noop_progress(), None) {
    Ok(_) => (),
    Err(_) => return Err(S5Error::new(ErrorKind::Internal, "Wallet-Sync")),
  };

  let decoded_psbt = match base64::decode(&psbt) {
    Ok(result) => result,
    Err(_) => return Err(S5Error::new(ErrorKind::Internal, "PSBT-Decode")),
  };
  let psbt_struct: PartiallySignedTransaction = match deserialize(&decoded_psbt) {
    Ok(result) => result,
    Err(_) => return Err(S5Error::new(ErrorKind::Internal, "PSBT-Deserialize")),
  };
  let tx = psbt_struct.extract_tx();
  let txid = match wallet.broadcast(tx){
    Ok(result) => result,
    Err(e) => return Err(S5Error::new(ErrorKind::Internal,&e.to_string())),
  };
  

  Ok(Txid {
    txid: txid.to_string(),
  })
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::config::WalletConfig;
  use bitcoin::network::constants::Network;

  #[test]
  fn test_send() {
    let xkey = "[db7d25b5/84'/1'/6']tpubDCCh4SuT3pSAQ1qAN86qKEzsLoBeiugoGGQeibmieRUKv8z6fCTTmEXsb9yeueBkUWjGVzJr91bCzeCNShorbBqjZV4WRGjz3CrJsCboXUe";
    let deposit_desc = format!("wpkh({}/0/*)", xkey);
    let node_address = "ssl://electrum.blockstream.info:60002";

    let config = WalletConfig::default(&deposit_desc, node_address).unwrap();
    let xkey = "[db7d25b5/84'/1'/6']tprv8fWev2sCuSkVWYoNUUSEuqLkmmfiZaVtgxosS5jRE9fw5ejL2odsajv1QyiLrPri3ppgyta6dsFaoDVCF4ZdEAR6qqY4tnaosujsPzLxB49";
    let deposit_desc = format!("wpkh({}/0/*)", xkey);

    let sign_config = WalletConfig::default(&deposit_desc, node_address).unwrap();
    let to = "mkHS9ne12qx9pS9VojpwU5xtRd4T7X7ZUt";
    let amount = 5_000;
    let fee_rate = 2.1;

    let psbt_origin = build(config, to, Some(amount), fee_rate,false);
    println!("{:#?}", psbt_origin);
    let decoded = decode(Network::Testnet, &psbt_origin.clone().unwrap().psbt);
    println!("Decoded: {:#?}", decoded.clone().unwrap());
    // assert_eq!(decoded.unwrap()[0].value, amount);
    let signed = sign(sign_config, &psbt_origin.unwrap().psbt);
    println!("{:#?}", signed.clone().unwrap());
    assert_eq!(signed.clone().unwrap().is_finalized, true);
    // let broadcasted = broadcast(config, &signed.unwrap().psbt);
    // println!("{:#?}",broadcasted.clone().unwrap());
    // assert_eq!(broadcasted.clone().unwrap().txid.len(), 64);
  }
  use bdk::electrum_client::{Client};
  use bdk::blockchain::{noop_progress, ElectrumBlockchain};

  #[test] #[ignore]
  fn test_build_absolute_and_rate(){
    let xkey = "[db7d25b5/84'/1'/6']tpubDCCh4SuT3pSAQ1qAN86qKEzsLoBeiugoGGQeibmieRUKv8z6fCTTmEXsb9yeueBkUWjGVzJr91bCzeCNShorbBqjZV4WRGjz3CrJsCboXUe";
    let deposit_desc = format!("wpkh({}/0/*)", xkey);
    let change_desc = deposit_desc.replace("/0/*","/1/*");
    let to_address = Address::from_str("mkHS9ne12qx9pS9VojpwU5xtRd4T7X7ZUt").unwrap();
    let amount = 5_000;
    let fee_rate = FeeRate::from_sat_per_vb(21.1);
    let client = Client::new("ssl://electrum.blockstream.info:60002").unwrap();

    let wallet = Wallet::new(
      &deposit_desc,
      Some(&change_desc),
      Network::Testnet,
      MemoryDatabase::default(),
      ElectrumBlockchain::from(client),
    ).unwrap();
    wallet.sync(noop_progress(), None).unwrap();
  
    let (psbt, details) = {
      let mut builder = wallet.build_tx();
      builder
      .enable_rbf()
      .add_recipient(to_address.script_pubkey(),amount)
      .fee_rate(fee_rate);
      builder.finish().unwrap()
    };

    let transaction:Transaction = psbt.extract_tx();
    let size = transaction.get_size();
    let fee_absolute = fee_rate.fee_vb(size);
    assert_eq!(fee_absolute,details.fee.unwrap());
  }
}
