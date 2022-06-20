use crate::config::WalletConfig;
use crate::e::{ErrorKind, S5Error};
use bdk::database::MemoryDatabase;
use bdk::wallet::AddressIndex::Peek;
use bdk::Wallet;
use serde::{Deserialize, Serialize};
use std::ffi::CString;
use std::os::raw::c_char;

/// FFI Output
#[derive(Serialize, Deserialize, Debug)]
pub struct WalletAddress {
    pub address: String,
}
impl WalletAddress {
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

pub fn generate(config: WalletConfig, index: u32) -> Result<WalletAddress, S5Error> {
    let wallet = match Wallet::new(
        &config.deposit_desc,
        Some(&config.change_desc),
        config.network,
        MemoryDatabase::default(),
    ) {
        Ok(result) => result,
        Err(e) => return Err(S5Error::new(ErrorKind::Internal, &e.to_string())),
    };
    match wallet.get_address(Peek(index)) {
        Ok(address) => Ok(WalletAddress {
            address: address.to_string(),
        }),
        Err(e) => Err(S5Error::new(ErrorKind::Internal, &e.to_string())),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_segwit_address() {
        let xkey = "[db7d25b5/84'/1'/6']tpubDCCh4SuT3pSAQ1qAN86qKEzsLoBeiugoGGQeibmieRUKv8z6fCTTmEXsb9yeueBkUWjGVzJr91bCzeCNShorbBqjZV4WRGjz3CrJsCboXUe";
        let descriptor = format!("wpkh({}/*)", xkey);
        let config = WalletConfig::new_offline(&descriptor).unwrap();
        let address0 = generate(config, 0).unwrap();
        assert_eq!(
            "tb1q093gl5yxww0hlvlkajdmf8wh3a6rlvsdk9e6d3".to_string(),
            address0.address
        );
        let config = WalletConfig::new_offline(&descriptor).unwrap();
        let address1 = generate(config, 1).unwrap();
        assert_eq!(
            "tb1qzdwqxt8l2s47vl4fp4ft6w67fcxel4qf5j96ld".to_string(),
            address1.address
        );
    }

    // fn test_raft_address(){
    //   let user = "[db7d25b5/84'/1'/6']tpubDCCh4SuT3pSAQ1qAN86qKEzsLoBeiugoGGQeibmieRUKv8z6fCTTmEXsb9yeueBkUWjGVzJr91bCzeCNShorbBqjZV4WRGjz3CrJsCboXUe";
    //   let custodian = "[66a0c105/84'/1'/5']tpubDCKvnVh6U56wTSUEJGamQzdb3ByAc6gTPbjxXQqts5Bf1dBMopknipUUSmAV3UuihKPTddruSZCiqhyiYyhFWhz62SAGuC3PYmtAafUuG6R";

    // }
}
