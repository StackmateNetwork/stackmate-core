use crate::e::{ErrorKind, S5Error};

use std::ffi::CString;
use std::os::raw::c_char;

use serde::{Deserialize, Serialize};

use crate::config::WalletConfig;
use bdk::blockchain::Blockchain;

#[derive(Serialize, Deserialize, Debug)]
pub struct BlockHeight {
  pub height: u32,
}
impl BlockHeight {
  pub fn c_stringify(&self) -> *mut c_char {
    let stringified = match serde_json::to_string(&self) {
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

pub fn get_height(config: WalletConfig) -> Result<BlockHeight, S5Error> {
  let height = match config.client.get_height() {
    Ok(result) => result,
    Err(e) => return Err(S5Error::new(ErrorKind::Internal, &e.to_string())),
  };
  Ok(BlockHeight { height })
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::config::DEFAULT_MAINNET_NODE;

  #[test]
  fn test_get_height() {
    let dummy_desc = "xprv/0/*";
    let config = WalletConfig::new(&dummy_desc, DEFAULT_MAINNET_NODE, None).unwrap();
    let height = get_height(config).unwrap();
    assert!(height.height>50000);
  }
}
