use std::str::FromStr;
use crate::e::{ErrorKind, S5Error};
// use crate::key::seed::MasterKey;
// use crate::key::derivation::ChildKeys;
// use crate::wallet::policy::WalletPolicy;
// use crate::config::WalletConfig;
use bdk::descriptor::Descriptor;
use bip39::{Language, Mnemonic};
use bitcoin::secp256k1::Secp256k1;
use std::ffi::CString;
use std::os::raw::c_char;
use serde::{Deserialize, Serialize};

#[derive(Deserialize,Serialize,Debug,PartialEq)]
pub enum RecoveryOption{
    MnemonicPhrase(String),
    Descriptor(String),
    None
}

impl FromStr for RecoveryOption {
    type Err = S5Error;
    fn from_str(dump: &str)->Result<RecoveryOption,Self::Err>{
        let trim_dump = dump.trim_end().trim_start();
        // check for descriptor
        {
            let is_descriptor = trim_dump.starts_with("wpkh(") || trim_dump.starts_with("wsh(");
            let trim_descriptor: String = trim_dump.chars().filter(|c| !c.is_whitespace()).collect();
            println!("{}",trim_descriptor);
            if is_descriptor {
                let secp = Secp256k1::new();
                match Descriptor::parse_descriptor(&secp,&trim_descriptor){
                    Ok(_)=> {
                        return Ok(RecoveryOption::Descriptor(trim_descriptor.to_string()))
                    },
                    Err(e)=> return Err(S5Error::new(ErrorKind::Input, &e.to_string()))
                };
            };
        }
        // check for mnemonic
        {
            let possible_mnemonic = trim_dump
            .split_whitespace()
            .collect::<Vec<&str>>();

            if possible_mnemonic.len() >= 12 {
                match Mnemonic::parse_in(Language::English, trim_dump.to_string()) {
                    Ok(_) => Ok(RecoveryOption::MnemonicPhrase(trim_dump.to_string())),
                    Err(_)=> Err(S5Error::new(ErrorKind::Input, "Looks like a bad mnemonic phrase!"))
                }
            }
            else{
                Ok(RecoveryOption::None)
            }
        }
       
    }
}

impl RecoveryOption{
    pub fn to_string(&self)->String{
        match self{
            RecoveryOption::Descriptor(desc)=>format!("descriptor:{}",desc),
            RecoveryOption::MnemonicPhrase(phrase)=>format!("mnemonic:{}",phrase),
            RecoveryOption::None=>"None".to_string()
        }
    }
    pub fn c_stringify(&self) -> *mut c_char {
        CString::new(self.to_string()).unwrap().into_raw()
      }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn try_recover_these(){
    let attempts: Vec<&str> = [
        "super strong bat",//invalid
        "wpkh([db7d25b5/84'/1'/6']tprv8fWev2sCuSkVWYoNUUSEuqLkmmfiZaVtgxosS5jRE9fw5ejL2odsajv1QyiLrPri3ppgyta6dsFaoDVCF4ZdEAR6qqY4tnaosujsPzLxB49/*)",//valid with no funds
        "wpkh([db7d25b5/84'/1'/6']WRONGKEY8fWev2sCuSkVWYoNUUSEuqLkmmfiZaVtgxosS5jRE9fw5ejL2odsajv1QyiLrPri3ppgyta6dsFaoDVCF4ZdEAR6qqY4tnaosujsPzLxB49/*)",//invalid
        "wsh(fail[db7d25b5/84'/1'/6']tprv8fWev2sCuSkVWYoNUUSEuqLkmmfiZaVtgxosS5jRE9fw5ejL2odsajv1QyiLrPri3ppgyta6dsFaoDVCF4ZdEAR6qqY4tnaosujsPzLxB49/*)",//invalid
        "transfer spare party divorce screen used pole march warfare another balance find",//valid with funds
        "church spare party divorce screen used pole march warfare another balance find",// invalid with no funds
    ].to_vec();

    let options: Vec<RecoveryOption> = attempts.clone().into_iter().map(|attempt| {
      RecoveryOption::from_str(attempt).unwrap_or(RecoveryOption::None)
    }).collect();

    println!("{:#?}",options);

    assert_eq!(options[0],RecoveryOption::None);
    assert_eq!(options[1],RecoveryOption::Descriptor(attempts[1].to_string()));
    assert_eq!(options[2],RecoveryOption::None);
    assert_eq!(options[3],RecoveryOption::None);
    assert_eq!(options[4],RecoveryOption::MnemonicPhrase(attempts[4].to_string()));
    assert_eq!(options[5],RecoveryOption::None);

  }
}