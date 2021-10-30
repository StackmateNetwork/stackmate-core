use std::ffi::{CString};
use std::os::raw::c_char;

use serde::{Serialize,Deserialize};

use bdk::blockchain::{Blockchain};

use crate::config::{WalletConfig};
use crate::e::{S5Error,ErrorKind};


#[derive(Serialize,Deserialize,Debug)]
pub struct NetworkFee {
    pub fee: f32,
}
impl NetworkFee{
  pub fn c_stringify(&self)->*mut c_char{
    let stringified = match serde_json::to_string(self){
        Ok(result)=>result,
        Err(_)=>return CString::new("Error:JSON Stringify Failed. BAD NEWS! Contact Support.").unwrap().into_raw()
    };

    CString::new(stringified).unwrap().into_raw()
  }
}

pub fn estimate_sats_per_byte(config: WalletConfig, target: usize)->Result<NetworkFee, S5Error>{

  let fee = match config.client.estimate_fee(target){
    Ok(result)=>result,
    Err(e)=>return Err(S5Error::new(ErrorKind::Internal,&e.to_string()))
  };
    Ok(NetworkFee{
      fee: fee.as_sat_vb()
    })
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::config::{DEFAULT_MAINNET_NODE};
  #[test]
  fn test_estimate_fee() {
    let dummy_desc = "xprv/0/*";
    let config = WalletConfig::default(&dummy_desc,DEFAULT_MAINNET_NODE).unwrap();
    let network_fee = estimate_sats_per_byte(config,1).unwrap();
    println!("{:#?}",network_fee);
  }


}