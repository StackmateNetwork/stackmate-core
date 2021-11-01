use std::ffi::CString;
use std::os::raw::c_char;

use serde::{Deserialize, Serialize};

use bdk::blockchain::Blockchain;
use bdk::FeeRate;

use crate::config::WalletConfig;
use crate::e::{ErrorKind, S5Error};

/// FFI Output
#[derive(Serialize, Deserialize, Debug)]
pub struct NetworkFee {
  pub rate: f32,
  pub absolute: Option<u64>,
}
impl NetworkFee {
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

pub fn estimate_sats_per_byte(config: WalletConfig, target: usize) -> Result<NetworkFee, S5Error> {
  let fee = match config.client.estimate_fee(target) {
    Ok(result) => result,
    Err(e) => return Err(S5Error::new(ErrorKind::Internal, &e.to_string())),
  };
  Ok(NetworkFee {
    rate: fee.as_sat_vb(),
    absolute: None,
  })
}

pub fn get_absolute(fee_rate: f32, weight: usize) -> NetworkFee {
  NetworkFee {
    rate: fee_rate,
    absolute: Some(FeeRate::from_sat_per_vb(fee_rate).fee_wu(weight)),
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::config::DEFAULT_MAINNET_NODE;
  #[test]
  fn test_estimate_fee() {
    let dummy_desc = "xprv/0/*";
    let config = WalletConfig::new(&dummy_desc, DEFAULT_MAINNET_NODE, None).unwrap();
    let network_fee = estimate_sats_per_byte(config, 1).unwrap();
    println!("{:#?}", network_fee);
  }
}
