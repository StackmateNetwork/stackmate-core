// use crate::e::{S5Error,ErrorKind};

// use std::ffi::{CString};
// use std::os::raw::c_char;

// use serde::{Serialize,Deserialize};

// use bdk::electrum_client::Client;
// use bdk::blockchain::{Blockchain};
// use bdk::blockchain::electrum::{ElectrumBlockchain};

// #[derive(Serialize,Deserialize,Debug)]
// pub struct BlockHeight {
//     pub height: u32,
// }
// impl BlockHeight{
//   pub fn _c_stringify(&self)->*mut c_char{
//     let stringified = match serde_json::to_string(self.clone()){
//         Ok(result)=>result,
//         Err(_)=>return CString::new("Error:JSON Stringify Failed. BAD NEWS! Contact Support.").unwrap().into_raw()
//     };

//     CString::new(stringified).unwrap().into_raw()
//   }
// }

// pub fn _get_height(node_address: &str)->Result<BlockHeight, S5Error>{
//   let client = match Client::new(node_address) {
//     Ok(result) => result,
//     Err(_) => return Err(S5Error::new(ErrorKind::OpError,"Node-Address-Connection"))
//   };    
//   let blockchain:ElectrumBlockchain = ElectrumBlockchain::from(client);
//   let height = blockchain.get_height().unwrap();
//     Ok(BlockHeight{
//       height: height
//     })
// }

// #[cfg(test)]
// mod tests {
//   use super::*;

//   #[test]
//   fn test_get_height() {
//     let node_address = "ssl://electrum.blockstream.info:50002"; // mainnet port used
//     let network_fee = _get_height(node_address).unwrap();
//     println!("{:#?}",network_fee);
//   }

// }