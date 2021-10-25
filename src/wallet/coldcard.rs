use serde_derive::{Serialize,Deserialize};
use std::fs::File;
use crate::e::{S5Error,ErrorKind};

#[derive(Default, Debug, Clone, PartialEq, Serialize,Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ColdCardKeys {
    pub chain: String,
    pub xpub: String,
    pub xfp: String,
    pub account: i64,
    pub bip49: Bip49,
    pub bip44: Bip44,
    pub bip84: Bip84,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Bip49 {
    pub xpub: String,
    pub first: String,
    pub deriv: String,
    pub xfp: String,
    pub name: String,
    #[serde(rename = "_pub")]
    pub ypub: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Bip44 {
    pub xpub: String,
    pub first: String,
    pub deriv: String,
    pub xfp: String,
    pub name: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Bip84 {
    pub xpub: String,
    pub first: String,
    pub deriv: String,
    pub xfp: String,
    pub name: String,
    #[serde(rename = "_pub")]
    pub zpub: String,
}

impl ColdCardKeys{
  pub fn _from_json_file(path: &str)->Result<ColdCardKeys,S5Error>{
    let file = match File::open(path){
      Ok(file)=>file,
      Err(_)=> return Err(S5Error::new(ErrorKind::OpError,"File-Open-Cold-Card-Json."))
    };
    
    match serde_json::from_reader(file){
      Ok(result)=>Ok(result),
      Err(_)=>Err(S5Error::new(ErrorKind::OpError,"JSON-File-To-Struct"))
    }

    
  }
}


#[cfg(test)]

mod tests {
  use super::*;
  use std::path::PathBuf;
  use crate::wallet::address;
  use crate::config::{WalletConfig, DEFAULT_MAINNET_NODE}; 

  #[test] #[ignore]
  fn test_coldcard_watcher() {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("resources/cc.json");
    let cckeys = ColdCardKeys::_from_json_file(&path.to_str().unwrap()).unwrap();
  
    let key_source_84 = cckeys.bip84.deriv.replace("m",&cckeys.xfp.to_lowercase());
    let bip84_deposit_desc = format!("wpkh([{}]{}/0/*)",key_source_84,cckeys.bip84.xpub);
    let config = WalletConfig::default(&bip84_deposit_desc,DEFAULT_MAINNET_NODE).unwrap();
    let bip84_first_address = address::generate(config,0).unwrap();
    assert_eq!(bip84_first_address.address,cckeys.bip84.first);

    let key_source_49 = cckeys.bip49.deriv.replace("m",&cckeys.xfp.to_lowercase());
    let bip49_deposit_desc = format!("sh(wpkh([{}]{}/0/*))",key_source_49,cckeys.bip49.xpub);
    let config = WalletConfig::default(&bip49_deposit_desc, DEFAULT_MAINNET_NODE).unwrap();

    let bip49_first_address = address::generate(config,0).unwrap();
    assert_eq!(bip49_first_address.address,cckeys.bip49.first);

    let key_source_44 = cckeys.bip49.deriv.replace("m",&cckeys.xfp.to_lowercase());
    let bip44_deposit_desc = format!("pkh([{}]{}/0/*)",key_source_44,cckeys.bip44.xpub);
    let config = WalletConfig::default(&bip44_deposit_desc,DEFAULT_MAINNET_NODE).unwrap();

    let bip44_first_address = address::generate(config,0).unwrap();
    assert_eq!(bip44_first_address.address,cckeys.bip44.first);
    
  }
}
