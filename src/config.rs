use crate::e::{ErrorKind, S5Error};

use bdk::blockchain::electrum::ElectrumBlockchainConfig;
use bdk::blockchain::rpc::{wallet_name_from_descriptor, Auth, RpcConfig};
use bdk::blockchain::any::{AnyBlockchain,AnyBlockchainConfig};
use bdk::blockchain::{ConfigurableBlockchain,ElectrumBlockchain, RpcBlockchain, Blockchain};
use bdk::electrum_client::{Error as ElectrumError};
use bdk::core_rpc::{Error as RpcError};
use bitcoin::network::constants::Network;
use bitcoin::secp256k1::Secp256k1;


pub struct WalletConfig {
  pub deposit_desc: String,
  pub change_desc: String,
  pub network: Network,
  pub client: AnyBlockchain,
}

pub const DEFAULT: &str = "default";
pub const DEFAULT_TESTNET_NODE: &str = "ssl://electrum.blockstream.info:60002";
pub const DEFAULT_MAINNET_NODE  : &str = "ssl://electrum.blockstream.info:50002";

impl WalletConfig {

  pub fn default(
    deposit_desc: &str,
    node_address: &str,
  ) -> Result<Self, S5Error> {

    let change_desc: &str = &deposit_desc.replace("/0/*", "/1/*");
    let network = if <&str>::clone(&deposit_desc).contains("xpub") || <&str>::clone(&deposit_desc).contains("xprv") {
      Network::Bitcoin
    } else {
      Network::Testnet
    };
    
    let node_address = if node_address.contains(DEFAULT){
      match network{
        Network::Bitcoin=>DEFAULT_MAINNET_NODE,
        _=>DEFAULT_TESTNET_NODE
      }
    }
    else{
      node_address
    };

    if node_address.contains("electrum") {
      let config = ElectrumBlockchainConfig {
        url: node_address.to_string(),
        socks5: None,
        retry: 1,
        timeout: Some(5),
        stop_gap: 1000,
      };
      let client = match create_blockchain_client(AnyBlockchainConfig::Electrum(config)) {
        Ok(client)=>client,
        Err(e)=>return Err(S5Error::new(ErrorKind::Internal,&e.message))
      };

      Ok(WalletConfig {
        deposit_desc: deposit_desc.to_string(),
        change_desc: change_desc.to_string(),
        network,
        client,
      })
    } else if node_address.contains("http") {
      let parts: Vec<&str> = node_address.split("?auth=").collect();
      let auth = if parts[1].is_empty() {
        Auth::None
      } else {
        Auth::UserPass {
          username: parts[1].split(':').collect::<Vec<&str>>()[0].to_string(),
          password: parts[1].split(':').collect::<Vec<&str>>()[1].to_string(),
        }
      };
      let wallet_name = match wallet_name_from_descriptor(
        deposit_desc,
        Some(change_desc),
        network,
        &Secp256k1::new(),
      ) {
        Ok(name) => name,
        Err(e) => return Err(S5Error::new(ErrorKind::Internal, &e.to_string())),
      };
     
      let config = RpcConfig {
        url: parts[0].to_string(),
        auth,
        network,
        wallet_name,
        skip_blocks: None,
      };
      let client = match create_blockchain_client(AnyBlockchainConfig::Rpc(config)){
        Ok(client)=>client,
        Err(e)=>return Err(S5Error::new(ErrorKind::Internal,&e.message))
      };

      Ok(WalletConfig {
        deposit_desc: deposit_desc.to_string(),
        change_desc: change_desc.to_string(),
        network,
        client
      })
    }
    else{
      Err(S5Error::new(ErrorKind::Internal, "Invalid Node Address."))
    }
  }
}

pub fn create_blockchain_client(config: AnyBlockchainConfig)->Result<AnyBlockchain,S5Error>{
  match config{
    AnyBlockchainConfig::Electrum(conf)=>{ 
      let client = match ElectrumBlockchain::from_config(&conf) {
        Ok(result) => result,
        Err(bdk_error) => {
          match bdk_error {
            bdk::Error::Electrum(electrum_error)=>match electrum_error{
              ElectrumError::IOError(c_error)=> return Err(S5Error::new(ErrorKind::Network, &c_error.to_string())),
              e_error=> return Err(S5Error::new(ErrorKind::Internal,&e_error.to_string()))
            }
            e_error=>{
              return Err(S5Error::new(ErrorKind::Internal, &e_error.to_string()))
            }
          }
        },
      };
      Ok(AnyBlockchain::Electrum(client))
    }
    AnyBlockchainConfig::Rpc(conf)=>{ 
      println!("{:#?}",conf);
      let client = match RpcBlockchain::from_config(&conf) {
        Ok(result) => result,
        Err(bdk_error) => {
          match bdk_error {
            bdk::Error::Rpc(rpc_error)=>match rpc_error{
              RpcError::Io(c_error)=> return Err(S5Error::new(ErrorKind::Network, &c_error.to_string())),
              r_error=> return Err(S5Error::new(ErrorKind::Internal, &r_error.to_string()))
            }
            r_error=>{
              return Err(S5Error::new(ErrorKind::Internal, &r_error.to_string()))
            }
          }
        },
      };
      Ok(AnyBlockchain::Rpc(client))
    }
  }
}

pub fn _check_client(network: Network, node_address: &str)->Result<bool,S5Error>{
  let client: AnyBlockchain = if node_address.contains("electrum") {
    let config = ElectrumBlockchainConfig {
      url: node_address.to_string(),
      socks5: None,
      retry: 1,
      timeout: Some(5),
      stop_gap: 1000,
    };
    match create_blockchain_client(AnyBlockchainConfig::Electrum(config)) {
      Ok(client)=>client,
      Err(e)=>return Err(S5Error::new(ErrorKind::Internal,&e.message))
    }

  } else if node_address.contains("http") {
    let parts: Vec<&str> = node_address.split("?auth=").collect();
    let auth = if parts[1].is_empty() {
      Auth::None
    } else {
      Auth::UserPass {
        username: parts[1].split(':').collect::<Vec<&str>>()[0].to_string(),
        password: parts[1].split(':').collect::<Vec<&str>>()[1].to_string(),
      }
    };
    
    let config = RpcConfig {
      url: parts[0].to_string(),
      auth,
      network,
      wallet_name: "ping".to_string(),
      skip_blocks: None,
    };
    
    match create_blockchain_client(AnyBlockchainConfig::Rpc(config)){
      Ok(client)=>client,
      Err(e)=>return Err(S5Error::new(ErrorKind::Internal,&e.message))
    }
  }
  else{
    return Err(S5Error::new(ErrorKind::Internal, "Invalid Node Address."))
  };

  match client.estimate_fee(1){
    Ok(_)=>Ok(true),
    Err(e)=>Err(S5Error::new(ErrorKind::Network,&e.to_string()))
  }
}


#[cfg(test)]
mod tests {
  use super::*;
  use crate::config::WalletConfig;
  use bitcoin::network::constants::Network;
  use bdk::blockchain::Blockchain;
  #[test]
  fn test_default_electrum_config() {
    let xkey = "[db7d25b5/84'/1'/6']tpubDCCh4SuT3pSAQ1qAN86qKEzsLoBeiugoGGQeibmieRUKv8z6fCTTmEXsb9yeueBkUWjGVzJr91bCzeCNShorbBqjZV4WRGjz3CrJsCboXUe";
    let deposit_desc = format!("wpkh({}/0/*)", xkey);

    let config = WalletConfig::default(&deposit_desc,DEFAULT_TESTNET_NODE).unwrap();
    match config.client{
      AnyBlockchain::Electrum(client)=>{
        let height = client.get_height().unwrap();
        println!("{:#?}",height);
        assert_eq!((height>2097921),true);
      },
      _=>println!("Should not reach.")
    };

    let change_desc = format!("wpkh({}/1/*)", xkey);
    let network = Network::Testnet;
    assert_eq!(config.change_desc,change_desc);
    assert_eq!(config.network,network);

  }

  #[test] #[ignore]
  fn test_local_rpc_config() {
    let xkey = "[db7d25b5/84'/1'/6']tpubDCCh4SuT3pSAQ1qAN86qKEzsLoBeiugoGGQeibmieRUKv8z6fCTTmEXsb9yeueBkUWjGVzJr91bCzeCNShorbBqjZV4WRGjz3CrJsCboXUe";
    let deposit_desc = format!("wpkh({}/0/*)", xkey);
    let node_address = "http://172.18.0.2:18332?auth=satsbank:typercuz";
    let config = WalletConfig::default(&deposit_desc,node_address).unwrap();

    match config.client{
      AnyBlockchain::Rpc(client)=>{
        let height = client.get_height().unwrap();
        println!("{:#?}",height);
        assert_eq!((height>2097921),true);
      },
      _=>println!("Should not reach.")
    };

    let change_desc = format!("wpkh({}/1/*)", xkey);
    let network = Network::Testnet;
    assert_eq!(config.change_desc,change_desc);
    assert_eq!(config.network,network);
    // println!("Connect a local node and then remove ignore macro.")
  }

  #[test]

  fn test_config_errors(){
    let dummy_desc = "xprv/0/*";
    let node_address = "ssl://electrum.blockstream.info:5002";
    let config_error = WalletConfig::default(&dummy_desc,node_address).err().unwrap();
    println!("{:#?}",config_error);

  }
}
