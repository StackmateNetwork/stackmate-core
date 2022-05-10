use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::ffi::CString;
use std::os::raw::c_char;
use std::str::FromStr;
use bdk::database::MemoryDatabase;
use bdk::descriptor::Descriptor;
use bdk::miniscript::DescriptorTrait;
use bdk::blockchain::Blockchain;
use bdk::Error;
use bdk::{KeychainKind, SignOptions,SyncOptions, Wallet};
use bitcoin::base64;
use bitcoin::blockdata::transaction::Transaction;
use bitcoin::consensus::deserialize;
use bitcoin::network::constants::Network;
use bitcoin::util::address::Address;
use bdk::bitcoin::util::psbt::PartiallySignedTransaction;
use crate::config::WalletConfig;
use crate::e::{ErrorKind, S5Error};
use bdk::bitcoin::Txid;
/// FFI Output
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct WalletPSBT {
  pub psbt: String,
  pub is_finalized: bool,
}
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

#[derive(Deserialize,Clone)]
pub struct TxOutput {
  pub address: String,
  pub amount: Option<u64>,
}

pub type TxOutputs = Vec<TxOutput>;

impl TxOutput {
  /// outputs as a str is address:amount,address:amount,address:amount
  pub fn vec_from_str(str: &str) -> Result<TxOutputs, S5Error> {
    let mut outputs: Vec<TxOutput> = Vec::new();
    for output in str.split(",") {
      let mut output_split = output.split(":");
      let address = match output_split.next() {
        Some(addr) => addr,
        None => return Err(S5Error::new(ErrorKind::Input, "Invalid tx outputs string")),
      };
      let amount = match output_split.next() {
        Some(amount) => match amount.parse::<u64>() {
          Ok(amount) => amount,
          Err(_) => return Err(S5Error::new(ErrorKind::Input, "Invalid tx amount")),
        },
        None => return Err(S5Error::new(ErrorKind::Input, "Invalid tx outputs string")),
      };

      outputs.push(TxOutput {
        address: address.to_string(),
        amount: Some(amount),
      });
    }
    Ok(outputs)
  }
  pub fn vec_from_json_str(str: &str) -> Result<TxOutputs, S5Error> {
    let outputs: TxOutputs = match serde_json::from_str(str) {
      Ok(result) => result,
      Err(_) => return Err(S5Error::new(ErrorKind::Input, "Invalid tx outputs string")),
    };
    Ok(outputs)
  }
}

#[derive(Deserialize,Clone)]
pub struct PolicyPath{
  pub id: String,
  pub path: Vec<usize>
}

impl PolicyPath{
  pub fn to_btreemap(&self) -> BTreeMap<String, Vec<usize>>{
    let mut map = BTreeMap::new();
    map.insert(self.id.clone(), self.path.clone());
    map
  }
  pub fn from_json_str(str: &str) -> Result<PolicyPath, S5Error> {
    let policy_path: PolicyPath = match serde_json::from_str(str) {
      Ok(result) => result,
      Err(_) => return Err(S5Error::new(ErrorKind::Input, "Invalid policy path.")),
    };
    Ok(policy_path)
  }
}

pub fn build(
  config: WalletConfig,
  outputs: Vec<TxOutput>,
  fee_absolute: u64,
  policy_path: Option<BTreeMap<String, Vec<usize>>>,
  sweep: bool,
) -> Result<WalletPSBT, S5Error> {
  
  let wallet = match Wallet::new(
    &config.deposit_desc,
    Some(&config.change_desc),
    config.network,
    MemoryDatabase::default(),
  ) {
    Ok(result) => result,
    Err(_) => return Err(S5Error::new(ErrorKind::Internal, "Wallet-Initialization")),
  };
  match wallet.sync(&config.client.unwrap(),SyncOptions::default()) {
    Ok(_) => (),
    Err(_) => return Err(S5Error::new(ErrorKind::Internal, "Wallet-Sync")),
  };

  let outputs = match outputs
    .iter()
    .map(|output| {
      let address = match Address::from_str(&output.address) {
        Ok(result) => result,
        Err(_) => return Err(S5Error::new(ErrorKind::Input, "Invalid Address")),
      };
      let amount = match output.amount {
        Some(result) => result,
        None => return Err(S5Error::new(ErrorKind::Input, "Invalid Amount")),
      };
      Ok((address, amount))
    })
    .collect::<Result<Vec<(Address, u64)>, S5Error>>()
  {
    Ok(result) => result,
    Err(_) => return Err(S5Error::new(ErrorKind::Input, "Invalid Output Set")),
  };

  let (psbt, _) = {
    let mut builder = wallet.build_tx();
    
    if sweep {
      builder
        .drain_wallet()
        .drain_to(outputs[0].0.script_pubkey());
    } else {
      outputs.iter().for_each(|(address, amount)| {
        builder.add_recipient(address.script_pubkey(), *amount);
      });
    }
    if policy_path.is_some() {
      builder.policy_path(policy_path.clone().unwrap(), KeychainKind::External);
      builder.policy_path(policy_path.unwrap(), KeychainKind::Internal);
    }

    builder.enable_rbf();
    builder.fee_absolute(fee_absolute);
    match builder.finish() {
      Ok(result) => result,
      Err(e) => {
        println!("{:?}", e);
        return match e {
          Error::SpendingPolicyRequired(_) => {
            Err(S5Error::new(ErrorKind::Input, "Spending Policy Required"))
          }
          _ => Err(S5Error::new(ErrorKind::Internal, "Transaction-Build")),
        };
      }
    }
  };

  Ok(WalletPSBT {
    psbt: psbt.to_string(),
    is_finalized: false,
  })
}
pub fn build_fee_bump(
  config: WalletConfig,
  txid: &str,
  fee_absolute: u64,
) -> Result<WalletPSBT, S5Error> {
  
  let wallet = match Wallet::new(
    &config.deposit_desc,
    Some(&config.change_desc),
    config.network,
    MemoryDatabase::default(),
  ) {
    Ok(result) => result,
    Err(_) => return Err(S5Error::new(ErrorKind::Internal, "Wallet-Initialization")),
  };
  match wallet.sync(&config.client.unwrap(), SyncOptions::default()) {
    Ok(_) => (),
    Err(_) => return Err(S5Error::new(ErrorKind::Internal, "Wallet-Sync")),
  };

  let txid = match Txid::from_str(txid){
    Ok(result)=>result,
    Err(e)=> return Err(S5Error::new(ErrorKind::Input, "Txid")),
  };

  let (psbt, _) = {
    let mut builder = match wallet.build_fee_bump(txid){
      Ok(result)=>result,
      Err(e)=>return Err(S5Error::new(ErrorKind::Wallet, &e.to_string())),
    };
    builder.fee_absolute(fee_absolute);
    match builder.finish() {
      Ok(result) => result,
      Err(e) => {
        println!("{:?}", e);
        return match e {
          Error::SpendingPolicyRequired(_) => {
            Err(S5Error::new(ErrorKind::Input, "Spending Policy Required"))
          }
          e => Err(S5Error::new(ErrorKind::Internal, &e.to_string())),
        };
      }
    }
  };

  Ok(WalletPSBT {
    psbt: psbt.to_string(),
    is_finalized: false,
  })
}

#[derive(Serialize, Debug, Clone)]
pub struct DecodedTxIO {
  value: u64,
  to: String,
}

#[derive(Serialize, Debug, Clone)]
pub struct DecodedTx {
  pub outputs: Vec<DecodedTxIO>,
  // pub weight: usize,
  // pub satisfaction_weight: usize
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

  let outputs = &psbt_struct.global.unsigned_tx.output;
  let mut decoded_outputs: Vec<DecodedTxIO> = vec![];
  let mut total_out_value = 0;
  for output in outputs {
    total_out_value += output.value;
    decoded_outputs.push(DecodedTxIO {
      value: output.value,
      to: match Address::from_script(&output.script_pubkey, network) {
        Some(address) => address.to_string(),
        None => "None".to_string(),
      },
    });
  }
  let inputs = &psbt_struct.inputs;
  let mut total_in_value = 0;
  for input in inputs {
    // let witness_utxo = input.witness_utxo.clone();
    total_in_value += input.witness_utxo.clone().unwrap().value;
  }

  decoded_outputs.push(DecodedTxIO {
    value: total_in_value - total_out_value,
    to: "miner".to_string(),
  });
  Ok(DecodedTx {
    outputs: decoded_outputs,
    // weight: weight + outputs.len() * 76,
  })
}

/// FFI Output
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TransactionWeight {
  pub weight: usize,
}

impl TransactionWeight {
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

pub fn get_weight(deposit_desc: &str, psbt: &str) -> Result<TransactionWeight, S5Error> {
  let decoded_psbt = match base64::decode(psbt) {
    Ok(psbt) => psbt,
    Err(_) => return Err(S5Error::new(ErrorKind::Internal, "Base64-Decode")),
  };

  let psbt_struct: PartiallySignedTransaction = match deserialize(&decoded_psbt) {
    Ok(psbt) => psbt,
    Err(_) => return Err(S5Error::new(ErrorKind::Internal, "Deserialize-Error")),
  };

  let transaction: Transaction = psbt_struct.extract_tx();
  let desc = Descriptor::<String>::from_str(deposit_desc).unwrap();
  let satisfaction_weight = desc.max_satisfaction_weight().unwrap();

  Ok(TransactionWeight {
    weight: transaction.get_weight() + satisfaction_weight,
  })
}

pub fn sign(config: WalletConfig, psbt: &str) -> Result<WalletPSBT, S5Error> {
  let wallet = match Wallet::new(
    &config.deposit_desc,
    Some(&config.change_desc),
    config.network,
    MemoryDatabase::default(),
  ) {
    Ok(result) => result,
    Err(_) => return Err(S5Error::new(ErrorKind::Internal, "Wallet-Initialization")),
  };
  let mut final_psbt: PartiallySignedTransaction = match deserialize(&base64::decode(psbt).unwrap()) {
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
pub struct TxidResponse {
  pub txid: String,
}
impl TxidResponse {
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

pub fn broadcast(config: WalletConfig, psbt: &str) -> Result<TxidResponse, S5Error> {

  let decoded_psbt = match base64::decode(&psbt) {
    Ok(result) => result,
    Err(_) => return Err(S5Error::new(ErrorKind::Internal, "PSBT-Decode")),
  };
  let psbt_struct: PartiallySignedTransaction = match deserialize(&decoded_psbt) {
    Ok(result) => result,
    Err(_) => return Err(S5Error::new(ErrorKind::Internal, "PSBT-Deserialize")),
  };
  let tx = psbt_struct.extract_tx();
  let txid = match config.client.unwrap().broadcast(&tx) {
    Ok(result) => result,
    Err(e) => return Err(S5Error::new(ErrorKind::Internal, &e.to_string())),
  };
  Ok(TxidResponse {
    txid: "success".to_string(),
  })
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::config::WalletConfig;
  use bitcoin::network::constants::Network;

  #[test]
  fn tx_outputs_deserialize(){
    let stringified = "[{\"address\":\"adssd\",\"amount\":1232},{\"address\":\"gaffs\",\"amount\":9232}]";
    let structured_from_json: TxOutputs = TxOutput::vec_from_json_str(stringified).unwrap();
    let str_format = "adssd:1232,gaffs:9232";
    let structured_from_str: TxOutputs = TxOutput::vec_from_str(str_format).unwrap();
    assert!(structured_from_json[1].address==structured_from_str[1].address);
    assert!(structured_from_json[1].address=="gaffs");
    assert!(structured_from_json[0].amount.is_some())
  }
  #[test]
  fn test_send() {
    let xkey = "[db7d25b5/84'/1'/6']tpubDCCh4SuT3pSAQ1qAN86qKEzsLoBeiugoGGQeibmieRUKv8z6fCTTmEXsb9yeueBkUWjGVzJr91bCzeCNShorbBqjZV4WRGjz3CrJsCboXUe";
    let descriptor = format!("wpkh({}/*)", xkey);
    let node_address = "ssl://electrum.blockstream.info:60002";
    let config = WalletConfig::new(&descriptor, node_address, None).unwrap();
    let xkey = "[db7d25b5/84'/1'/6']tprv8fWev2sCuSkVWYoNUUSEuqLkmmfiZaVtgxosS5jRE9fw5ejL2odsajv1QyiLrPri3ppgyta6dsFaoDVCF4ZdEAR6qqY4tnaosujsPzLxB49";
    let descriptor = format!("wpkh({}/*)", xkey);
    let sign_config = WalletConfig::new(&descriptor, node_address, None).unwrap();
    let broadcast_config = WalletConfig::new(&descriptor, node_address, None).unwrap();
    let bump_config = WalletConfig::new(&descriptor, node_address, None).unwrap();

    let to = "mkHS9ne12qx9pS9VojpwU5xtRd4T7X7ZUt";
    let amount = 5_000;
    let fee_absolute = 420;
    let output = TxOutput {
      address: to.to_string(),
      amount: Some(amount),
    };
    let psbt_origin = build(config, vec![output], fee_absolute, None, false);
    
    let decoded = decode(Network::Testnet, &psbt_origin.clone().unwrap().psbt);
    println!("Decoded: {:#?}", decoded.clone().unwrap());
    // assert_eq!(decoded.unwrap()[0].value, amount);
    let signed = sign(sign_config, &psbt_origin.clone().unwrap().psbt);
    println!("{:#?}", signed.clone().unwrap());
    assert_eq!(signed.clone().unwrap().is_finalized, true);
    // let broadcasted = broadcast(broadcast_config, &signed.unwrap().psbt).unwrap();
    // println!("{:#?}", psbt_origin.clone().unwrap());
    // let bumped  = build_fee_bump(bump_config,&broadcasted.txid, fee_absolute+1_000);

    // assert!(bumped.clone().unwrap().is_finalized);
  }

  #[test]

  fn test_get_weight() {
    let xkey = "[db7d25b5/84'/1'/6']tpubDCCh4SuT3pSAQ1qAN86qKEzsLoBeiugoGGQeibmieRUKv8z6fCTTmEXsb9yeueBkUWjGVzJr91bCzeCNShorbBqjZV4WRGjz3CrJsCboXUe";
    let descriptor = format!("wpkh({}/*)", xkey);
    let psbt = "cHNidP8BAHQBAAAAAf3cLERUN9+6X5+1yk3x9XzSCq1417WtB+gB5qNyj+xpAAAAAAD9////AnRxAQAAAAAAFgAUVyorkNVSCsiE4/7OspP52IwquzqIEwAAAAAAABl2qRQ0Sg9IyhUOwrkDgXZgubaLE6ZwJoisAAAAAAABAN4CAAAAAAEByvn9X3PvFqemGsrTv8ivAO07IOeRhBz7J0huqXJLfVgBAAAAAP7///8CoIYBAAAAAAAWABQTXAMs/1Qr5n6pDVK9O15ODZ/UCVZWjQAAAAAAFgAUIixaISTPlO8fwyT3hCL+An5+Km4CRzBEAiBFsQJfBur3eQgO5Vw+EvEgr2CagcVGXw9oYw3FOaMSSgIgch0CV+W3oRCKNBwxqiqIK0C5b1TsGk32HvNM+4Z7IksBIQNP/rsBHKbA98977TzmriFrOuO8hQjNg4ON3goI9/Uwjp0BIAABAR+ghgEAAAAAABYAFBNcAyz/VCvmfqkNUr07Xk4Nn9QJIgYD9WhlKKSeNh6567KTmyKrlitDWZOz/+mms7emVsWjGTsY230ltVQAAIABAACABgAAgAAAAAABAAAAACICAgHPrE7CShQkK90ApPF8xdr+8o7T/sHggOlZNOHIUft/GNt9JbVUAACAAQAAgAYAAIABAAAAAQAAAAAA";
    let expected_weight = 576;
    let tx_weight = get_weight(&descriptor, &psbt).unwrap();
    assert_eq!(tx_weight.weight, expected_weight);
  }
}
