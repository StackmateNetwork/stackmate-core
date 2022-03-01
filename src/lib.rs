/*
Developed by Stackmate India in 2021.
*/
//! # Stackmate
//! A set of composite functions that uses [rust-bitcoin](https://docs.rs/crate/bitcoin/0.27.1) & [bdk](bitcoindevkit.com) and exposes a simplified C interface to build descriptor based wallets.
//! ## Workflow
//! 1. Use key functions generate_master/import_master and derive a parent key at a hardened path with a variable account number. Currently purpose is fixed at 84' for Native-native only.
//! 2. Use extended key format to create string policies. More on [policies](http://bitcoin.sipa.be/miniscript/).
//! 3. Use the compile function to get a general descriptor (keys ending in /*).
//! 4. Use wallet functions by passing your descriptor and node_address as primary inputss.
//! 5. Electrum over ssl is the recommended way to interact with the wallet with format of 'ssl://electrum.blockstream.info:60002'.
//! 6. "default" can be used as a string for the node_address which will use Blockstream servers. Recommened client to use tor with this setting.
//! 7. Bitcoin-core RPC is supported but not advised unless on desktop where a node is connected to locally.
//! 8. Core RPC (currently) requies node_address to follow the format of 'https://address:port?auth=username:password'.
//! 9. Outputs of each function are JSON stringified native structs specified as 'FFI Outputs' in under module documentation.
//! 10. *Use every function in combination with cstring_free to free their output pointers. This will keep things safe.*
//!
//! ## Building a transaction
//! 1. Build a transaction with a default fixed fee of 1000 sats
//! 2. Get weight of the transaction for a given descriptor
//! 3. Use get absolute fee to get the fee needed to be paid for the transaction given variable fee rate and fixed weight.
//! 4. Build transaction with the absolute fee chosen, sign & broadcast.
//!
//!
//! ### Tor controls are in BETA. Use with caution.
//!

use std::alloc::System;

#[global_allocator]
static A: System = System;
use std::ffi::{CStr, CString};
use std::os::raw::c_char;
use std::str;
use bitcoin::network::constants::Network;

pub mod e;
use e::{ErrorKind, S5Error};

mod config;
use crate::config::{WalletConfig, DEFAULT, DEFAULT_MAINNET_NODE, DEFAULT_TESTNET_NODE};

pub mod key;
use crate::key::derivation;
use crate::key::seed;
use crate::key::ec;

pub mod wallet;
use crate::wallet::address;
use crate::wallet::history;
use crate::wallet::policy;
use crate::wallet::psbt;

pub mod network;
use crate::network::fees;
use crate::network::height;
use crate::network::tor;

/// Generates a mnemonic phrase of a given length. Defaults to 24 words.
/// A master xprv is created from the mnemonic and passphrase.
/// - *OUTPUT*
/// ```
/// MasterKey {
///   fingerprint: String,
///   mnemonic: String,
///   xprv: String,
/// }
/// ```
/// # Safety
/// - This function is unsafe because it dereferences and a returns raw pointer.
/// - ENSURE that output is passed into cstring_free(ptr: *mut c_char) after use.
#[no_mangle]
pub unsafe extern "C" fn generate_master(
    network: *const c_char,
    length: *const c_char,
    passphrase: *const c_char,
) -> *mut c_char {
    let input_cstr = CStr::from_ptr(length);
    let length: usize = match input_cstr.to_str() {
        Err(_) => 24,
        Ok(string) => match string.parse::<usize>() {
            Ok(l) => {
                if l == 12 || l == 24 {
                    l
                } else {
                    24
                }
            }
            Err(_) => 24,
        },
    };

    let passphrase_cstr = CStr::from_ptr(passphrase);
    let passphrase: &str = match passphrase_cstr.to_str() {
        Ok(string) => string,
        Err(_) => "",
    };

    let network_cstr = CStr::from_ptr(network);
    let network_str: &str = match network_cstr.to_str() {
        Ok(string) => string,
        Err(_) => "test",
    };
    let network = match network_str {
        "main" => Network::Bitcoin,
        "test" => Network::Testnet,
        _ => Network::Testnet,
    };

    match seed::generate(length, passphrase, network) {
        Ok(master_key) => master_key.c_stringify(),
        Err(e) => e.c_stringify(),
    }
}

/// Creates a master xprv given a mnemonic and passphrase.
/// - *OUTPUT*
/// ```
/// MasterKey {
///   fingerprint: String,
///   mnemonic: String,
///   xprv: String,
/// }
/// ```
/// # Safety
/// - This function is unsafe because it dereferences and a returns raw pointer.
/// - ENSURE that result is passed into cstring_free(ptr: *mut c_char) after use.
#[no_mangle]
pub unsafe extern "C" fn import_master(
    network: *const c_char,
    mnemonic: *const c_char,
    passphrase: *const c_char,
) -> *mut c_char {
    let input_cstr = CStr::from_ptr(mnemonic);
    let mnemonic: &str = match input_cstr.to_str() {
        Ok(string) => string,
        Err(_) => return S5Error::new(ErrorKind::Input, "Mnemonic").c_stringify(),
    };

    let passphrase_cstr = CStr::from_ptr(passphrase);
    let passphrase: &str = match passphrase_cstr.to_str() {
        Ok(string) => string,
        Err(_) => "",
    };

    let network_cstr = CStr::from_ptr(network);
    let network_str: &str = match network_cstr.to_str() {
        Ok(string) => string,
        Err(_) => "test",
    };
    let network = match network_str {
        "main" => Network::Bitcoin,
        "test" => Network::Testnet,
        _ => Network::Testnet,
    };

    match seed::import(mnemonic, passphrase, network) {
        Ok(master_key) => master_key.c_stringify(),
        Err(e) => e.c_stringify(),
    }
}

/// Derives hardened child keys from a master xprv.
/// Follows the BIP32 standard of m/purpose'/network'/account'.
/// Network path is inferred from the master xprv.
/// - *OUTPUT*
/// ```
/// ChildKeys {
///   fingerprint: String,
///   hardened_path: String,
///   xprv: String,
///   xpub: String,
/// }
/// ```
/// 
/// # Safety
/// - This function is unsafe because it dereferences and a returns raw pointer.
/// - ENSURE that result is passed into cstring_free(ptr: *mut c_char) after use.
#[no_mangle]
pub unsafe extern "C" fn derive_hardened(
    master_xprv: *const c_char,
    purpose: *const c_char,
    account: *const c_char,
) -> *mut c_char {
    let master_xprv_cstr = CStr::from_ptr(master_xprv);
    let master_xprv: &str = match master_xprv_cstr.to_str() {
        Ok(string) => string,
        Err(_) => return S5Error::new(ErrorKind::Input, "Master-Xprv").c_stringify(),
    };

    let purpose_cstr = CStr::from_ptr(purpose);
    let purpose:derivation::DerivationPurpose = match purpose_cstr.to_str() {
        Ok(string) => match string.parse::<usize>() {
            Ok(value) => {
                match value{
                    84=>derivation::DerivationPurpose::Native,
                    49=>derivation::DerivationPurpose::Compatible,
                    44=>derivation::DerivationPurpose::Legacy,
                    _=>derivation::DerivationPurpose::Native,
                }
            }
            Err(_) => derivation::DerivationPurpose::Native,
        },
        Err(_) => derivation::DerivationPurpose::Native,
    };

    let account_cstr = CStr::from_ptr(account);
    let account = match account_cstr.to_str() {
        Ok(string) => match string.parse::<u64>() {
            Ok(number) => number,
            Err(_) => 0,
        },
        Err(_) => 0,
    };

    match derivation::to_hardened_account(master_xprv, purpose, account) {
        Ok(result) => result.c_stringify(),
        Err(e) => e.c_stringify(),
    }
}
/// Derives child keys from a master xprv.
/// Allows passing a custom derivation path string.
/// # Safety
/// - This function is unsafe because it dereferences and a returns raw pointer.
/// - ENSURE that result is passed into cstring_free(ptr: *mut c_char) after use.
#[no_mangle]
pub unsafe extern "C" fn derive_path_str(
    master_xprv: *const c_char,
    derivation_path: *const c_char,
) -> *mut c_char {
    let master_xprv_cstr = CStr::from_ptr(master_xprv);
    let master_xprv: &str = match master_xprv_cstr.to_str() {
        Ok(string) => string,
        Err(_) => return S5Error::new(ErrorKind::Input, "Master-Xprv").c_stringify(),
    };

    let dp_cstr = CStr::from_ptr(derivation_path);
    let dp_str: &str = match dp_cstr.to_str() {
        Ok(string) => string,
        Err(_) => return S5Error::new(ErrorKind::Input, "Derivation-Path").c_stringify(),
    };
  
    match derivation::to_path_str(master_xprv, dp_str) {
        Ok(result) => result.c_stringify(),
        Err(e) => e.c_stringify(),
    }
}
/// Converts an xprv into EC keys with XOnlyPub..
/// # Safety
/// - This function is unsafe because it dereferences and a returns raw pointer.
/// - ENSURE that result is passed into cstring_free(ptr: *mut c_char) after use.
#[no_mangle]
pub unsafe extern "C" fn xprv_to_ec(
    master_xprv: *const c_char,
) -> *mut c_char {
    let master_xprv_cstr = CStr::from_ptr(master_xprv);
    let master_xprv: &str = match master_xprv_cstr.to_str() {
        Ok(string) => string,
        Err(_) => return S5Error::new(ErrorKind::Input, "Master-Xprv").c_stringify(),
    };

    let keypair = match ec::keypair_from_xprv_str(master_xprv) {
        Ok(result) => result,
        Err(_) => return S5Error::new(ErrorKind::Input, "Master-Xprv").c_stringify(),
    };
    ec::XOnlyPair::from_keypair(keypair).c_stringify()

}
/// Computes a Diffie Hellman shared secret
/// # Safety
/// - This function is unsafe because it dereferences and a returns raw pointer.
/// - ENSURE that result is passed into cstring_free(ptr: *mut c_char) after use.
#[no_mangle]
pub unsafe extern "C" fn shared_secret(
    local_priv: *const c_char,
    remote_pub: *const c_char
) -> *mut c_char {
    let local_priv_cstr = CStr::from_ptr(local_priv);
    let local_priv: &str = match local_priv_cstr.to_str() {
        Ok(string) => string,
        Err(_) => return S5Error::new(ErrorKind::Input, "Local-Private-Key").c_stringify(),
    };
    let remote_pub_cstr = CStr::from_ptr(remote_pub);
    let remote_pub: &str = match remote_pub_cstr.to_str() {
        Ok(string) => string,
        Err(_) => return S5Error::new(ErrorKind::Input, "Remote-Public-Key").c_stringify(),
    };

    match ec::compute_shared_secret_str(local_priv,remote_pub){
        Ok(result) => CString::new(result.to_string()).unwrap().into_raw(),
        Err(e) =>  e.c_stringify(),
    }
}
/// Signs a message using schnorr signature scheme 
/// # Safety
/// - This function is unsafe because it dereferences and a returns raw pointer.
/// - ENSURE that result is passed into cstring_free(ptr: *mut c_char) after use.
#[no_mangle]
pub unsafe extern "C" fn sign_message(
    message: *const c_char,
    seckey: *const c_char,
) -> *mut c_char {
    let seckey_cstr = CStr::from_ptr(seckey);
    let seckey: &str = match seckey_cstr.to_str() {
        Ok(string) => string,
        Err(_) => return S5Error::new(ErrorKind::Input, "Secret-Key").c_stringify(),
    };
    let message_cstr = CStr::from_ptr(message);
    let message: &str = match message_cstr.to_str() {
        Ok(string) => string,
        Err(_) => return S5Error::new(ErrorKind::Input, "Message").c_stringify(),
    };

    let keypair = match ec::keypair_from_seckey_str(seckey) {
        Ok(result) => result,
        Err(_) => return S5Error::new(ErrorKind::Input, "Master-Xprv").c_stringify(),
    };
    match ec::schnorr_sign(message, keypair){
        Ok(result) => CString::new(result.to_string()).unwrap().into_raw(),
        Err(e) =>  e.c_stringify(),
    }
}
/// Signs a message using schnorr signature scheme 
/// Private key extracted from extended private key.
/// # Safety
/// - This function is unsafe because it dereferences and a returns raw pointer.
/// - ENSURE that result is passed into cstring_free(ptr: *mut c_char) after use.
#[no_mangle]
pub unsafe extern "C" fn verify_signature(
    signature: *const c_char,
    message: *const c_char,
    pubkey: *const c_char
) -> *mut c_char {
    let signature_cstr = CStr::from_ptr(signature);
    let signature_str = match signature_cstr.to_str() {
        Ok(string) => string,
        Err(_) => return S5Error::new(ErrorKind::Input, "Signature").c_stringify(),
    };
 
    let message_cstr = CStr::from_ptr(message);
    let message_str: &str = match message_cstr.to_str() {
        Ok(string) => string,
        Err(_) => return S5Error::new(ErrorKind::Input, "Message").c_stringify(),
    };
    let pubkey_cstr = CStr::from_ptr(pubkey);
    let pubkey_str: &str = match pubkey_cstr.to_str() {
        Ok(string) => string,
        Err(_) => return S5Error::new(ErrorKind::Input, "Pubkey").c_stringify(),
    };

    match ec::schnorr_verify(signature_str, message_str, pubkey_str){
        Ok(result) => CString::new(result.to_string()).unwrap().into_raw(),
        Err(e) =>  e.c_stringify(),
    }
}
/// Compiles a policy into a descriptor of the specified script type.
/// Use wpkh for a single signature Native native wallet (default).
/// Use wsh for a scripted Native native wallet.
/// - *OUTPUT*
/// ```
/// WalletPolicy {
///   policy: String,
///   descriptor: String,
/// }
/// ```
/// # Safety
/// - This function is unsafe because it dereferences and a returns raw pointer.
/// - ENSURE that result is passed into cstring_free(ptr: *mut c_char) after use.
#[no_mangle]
pub unsafe extern "C" fn compile(policy: *const c_char, script_type: *const c_char) -> *mut c_char {
    let policy_cstr = CStr::from_ptr(policy);
    let policy_str: &str = match policy_cstr.to_str() {
        Ok(string) => string,
        Err(_) => return S5Error::new(ErrorKind::Input, "Policy").c_stringify(),
    };

    let script_type_cstr = CStr::from_ptr(script_type);
    let script_type_str: &str = match script_type_cstr.to_str() {
        Ok(string) => {
            if string == "wsh" || string == "wpkh" || string == "sh" || string == "sh-wsh" {
                string
            } else {
                "wpkh"
            }
        }
        Err(_) => "wpkh",
    };

    match policy::compile(policy_str, script_type_str) {
        Ok(result) => CString::new(result).unwrap().into_raw(),
        Err(e) => e.c_stringify(),
    }
}

/// Syncs to a remote node and fetches balance of a descriptor wallet.
/// - *OUTPUT*
/// ```
/// WalletBalance {
///   balance: u64
/// }
/// ```
/// # Safety
/// - This function is unsafe because it dereferences and a returns raw pointer.
/// - ENSURE that result is passed into cstring_free(ptr: *mut c_char) after use.
#[no_mangle]
pub unsafe extern "C" fn sync_balance(
    descriptor: *const c_char,
    node_address: *const c_char,
) -> *mut c_char {
    let descriptor_cstr = CStr::from_ptr(descriptor);
    let descriptor: &str = match descriptor_cstr.to_str() {
        Ok(string) => string,
        Err(_) => return S5Error::new(ErrorKind::Input, "Descriptor").c_stringify(),
    };

    let node_address_cstr = CStr::from_ptr(node_address);
    let node_address: &str = match node_address_cstr.to_str() {
        Ok(string) => {
            if string.contains("electrum") || string.contains("http") {
                string
            } else {
                DEFAULT
            }
        }
        Err(_) => DEFAULT,
    };

    let config = match WalletConfig::new(descriptor, node_address, None) {
        Ok(conf) => conf,
        Err(e) => return S5Error::new(ErrorKind::Internal, &e.message).c_stringify(),
    };
    match history::sync_balance(config) {
        Ok(result) => result.c_stringify(),
        Err(e) => e.c_stringify(),
    }
}

/// Syncs to a remote node and fetches history of a descriptor wallet.
/// - *OUTPUT*
/// ```
///  WalletHistory{
///    history: Vec<Transaction {
///      timestamp: u64,
///      height: u32,
///      verified: bool,
///      txid: String,
///      received: u64,
///      sent: u64,
///      fee: u64,
///    }>;
///  }
/// ```
/// # Safety
/// - This function is unsafe because it dereferences and a returns raw pointer.
/// - ENSURE that result is passed into cstring_free(ptr: *mut c_char) after use.
#[no_mangle]
pub unsafe extern "C" fn sync_history(
    descriptor: *const c_char,
    node_address: *const c_char,
) -> *mut c_char {
    let descriptor_cstr = CStr::from_ptr(descriptor);
    let descriptor: &str = match descriptor_cstr.to_str() {
        Ok(string) => string,
        Err(_) => return S5Error::new(ErrorKind::Input, "Descriptor").c_stringify(),
    };

    let node_address_cstr = CStr::from_ptr(node_address);
    let node_address: &str = match node_address_cstr.to_str() {
        Ok(string) => {
            if string.contains("electrum") || string.contains("http") {
                string
            } else {
                DEFAULT
            }
        }
        Err(_) => DEFAULT,
    };

    let config = match WalletConfig::new(descriptor, node_address, None) {
        Ok(conf) => conf,
        Err(e) => return S5Error::new(ErrorKind::Internal, &e.message).c_stringify(),
    };
    match history::sync_history(config) {
        Ok(result) => result.c_stringify(),
        Err(e) => e.c_stringify(),
    }
}

/// Gets a new address for a descriptor wallet at a given index.
/// Client must keep track of address indexes and ENSURE prevention of address reuse.
/// - *OUTPUT*
/// ```
/// WalletAddress {
///   address: String,
/// }
/// ```
/// # Safety
/// - This function is unsafe because it dereferences and a returns raw pointer.
/// - ENSURE that result is passed into cstring_free(ptr: *mut c_char) after use.
#[no_mangle]
pub unsafe extern "C" fn get_address(
    descriptor: *const c_char,
    index: *const c_char,
) -> *mut c_char {
    let descriptor_cstr = CStr::from_ptr(descriptor);
    let descriptor: &str = match descriptor_cstr.to_str() {
        Ok(string) => string,
        Err(_) => return S5Error::new(ErrorKind::Input, "Descriptor").c_stringify(),
    };



    let config = match WalletConfig::new_offline(descriptor) {
        Ok(conf) => conf,
        Err(e) => return S5Error::new(ErrorKind::Internal, &e.message).c_stringify(),
    };

    let index_cstr = CStr::from_ptr(index);
    let address_index: u32 = match index_cstr.to_str() {
        Ok(string) => match string.parse::<u32>() {
            Ok(i) => i,
            Err(_) => {
                return CString::new("Error: Address Index Input.")
                    .unwrap()
                    .into_raw()
            }
        },
        Err(_) => return S5Error::new(ErrorKind::Input, "Address-Index").c_stringify(),
    };

    match address::generate(config, address_index) {
        Ok(result) => result.c_stringify(),
        Err(e) => e.c_stringify(),
    }
}

/// Gets the current network fee (in sats/vbyte) for a given confirmation target.
/// - *OUTPUT*
/// ```  
///  NetworkFee {
///    rate: f32,
///    absolute: Option<u64>,
///  }
/// ```
/// # Safety
/// - This function is unsafe because it dereferences and a returns raw pointer.
/// - ENSURE that result is passed into cstring_free(ptr: *mut c_char) after use.
#[no_mangle]
pub unsafe extern "C" fn estimate_network_fee(
    network: *const c_char,
    node_address: *const c_char,
    conf_target: *const c_char,
) -> *mut c_char {
    let conf_target_cstr = CStr::from_ptr(conf_target);
    let conf_target_int: usize = match conf_target_cstr.to_str() {
        Ok(string) => string.parse::<usize>().unwrap_or(6),
        Err(_) => 6,
    };

    let network_cstr = CStr::from_ptr(network);
    let network: &str = match network_cstr.to_str() {
        Ok(string) => string,
        Err(_) => "test",
    };
    let network_enum = match network {
        "main" => Network::Bitcoin,
        _ => Network::Testnet,
    };
    let node_address_cstr = CStr::from_ptr(node_address);
    let node_address: &str = match node_address_cstr.to_str() {
        Ok(string) => {
            if string == DEFAULT {
                match network_enum {
                    Network::Bitcoin => DEFAULT_MAINNET_NODE,
                    _ => DEFAULT_TESTNET_NODE,
                }
            } else {
                string
            }
        }
        Err(_) => match network_enum {
            Network::Bitcoin => DEFAULT_MAINNET_NODE,
            _ => DEFAULT_TESTNET_NODE,
        },
    };

    let config = match WalletConfig::new("*", node_address, None) {
        Ok(conf) => conf,
        Err(e) => return S5Error::new(ErrorKind::Internal, &e.message).c_stringify(),
    };
    match fees::estimate_rate(config, conf_target_int) {
        Ok(result) => result.c_stringify(),
        Err(e) => e.c_stringify(),
    }
}

/// Converts a given fee_rate (in sats/vbyte) to absolute fee (in sats); given some transaction weight.
/// - *OUTPUT*
/// ```  
///  NetworkFee {
///    rate: f32,
///    absolute: Option<u64>,
///  }
/// ```
/// # Safety
/// - This function is unsafe because it dereferences and a returns raw pointer.
/// - ENSURE that result is passed into cstring_free(ptr: *mut c_char) after use.
#[no_mangle]
pub unsafe extern "C" fn fee_rate_to_absolute(
    fee_rate: *const c_char,
    weight: *const c_char,
) -> *mut c_char {
    let weight_cstr = CStr::from_ptr(weight);
    let weight_usize: usize = match weight_cstr.to_str() {
        Ok(string) => string.parse::<usize>().unwrap_or(250),
        Err(_) => 250,
    };

    let fee_rate_cstr = CStr::from_ptr(fee_rate);
    let fee_rate_f32: f32 = match fee_rate_cstr.to_str() {
        Ok(string) => string.parse::<f32>().unwrap_or(1.0),
        Err(_) => 1.0,
    };

    fees::get_absolute(fee_rate_f32, weight_usize).c_stringify()
}

/// Converts a given absolute_fee (in sats) to fee rate (in sats/vbyte); given some transaction weight.
/// - *OUTPUT*
/// ```  
///  NetworkFee {
///    rate: f32,
///    absolute: Option<u64>,
///  }
/// ```
/// # Safety
/// - This function is unsafe because it dereferences and a returns raw pointer.
/// - ENSURE that result is passed into cstring_free(ptr: *mut c_char) after use.
#[no_mangle]
pub unsafe extern "C" fn fee_absolute_to_rate(
    fee_absolute: *const c_char,
    weight: *const c_char,
) -> *mut c_char {
    let weight_cstr = CStr::from_ptr(weight);
    let weight_usize: usize = match weight_cstr.to_str() {
        Ok(string) => string.parse::<usize>().unwrap_or(250),
        Err(_) => 250,
    };

    let fee_absolute_cstr = CStr::from_ptr(fee_absolute);
    let fee_absolute_u64: u64 = match fee_absolute_cstr.to_str() {
        Ok(string) => string.parse::<u64>().unwrap_or(1000),
        Err(_) => 1000,
    };

    fees::get_rate(fee_absolute_u64, weight_usize).c_stringify()
}

/// Gets the weight of a transaction built with a given descriptor.
/// - *OUTPUT*
/// ```  
///  TransactionWeight {
///     weight: usize,
///  }
/// ```
/// # Safety
/// - This function is unsafe because it dereferences and a returns raw pointer.
/// - ENSURE that result is passed into cstring_free(ptr: *mut c_char) after use.
#[no_mangle]
pub unsafe extern "C" fn get_weight(descriptor: *const c_char, psbt: *const c_char) -> *mut c_char {
    let descriptor_cstr = CStr::from_ptr(descriptor);
    let descriptor: &str = match descriptor_cstr.to_str() {
        Ok(string) => string,
        Err(_) => return S5Error::new(ErrorKind::Input, "Descriptor").c_stringify(),
    };

    let psbt_cstr = CStr::from_ptr(psbt);
    let psbt: &str = match psbt_cstr.to_str() {
        Ok(string) => string,
        Err(_) => return S5Error::new(ErrorKind::Input, "PSBT-Input").c_stringify(),
    };

    match psbt::get_weight(descriptor, psbt) {
        Ok(result) => result.c_stringify(),
        Err(e) => e.c_stringify(),
    }
}

/// Builds a transaction for a given descriptor wallet.
/// If sweep is set to true, amount value is ignored and will default to None.
/// Set amount to 0 for sweep.
/// - *OUTPUT*
/// ```
///  WalletPSBT {
///    pub psbt: String,
///    pub is_finalized: bool,
///  }
/// ```
/// # Safety
/// - This function is unsafe because it dereferences and a returns raw pointer.
/// - ENSURE that result is passed into cstring_free(ptr: *mut c_char) after use.
#[no_mangle]
pub unsafe extern "C" fn build_tx(
    descriptor: *const c_char,
    node_address: *const c_char,
    to_address: *const c_char,
    amount: *const c_char,
    fee_absolute: *const c_char,
    sweep: *const c_char,
) -> *mut c_char {
    let descriptor_cstr = CStr::from_ptr(descriptor);
    let descriptor: &str = match descriptor_cstr.to_str() {
        Ok(string) => string,
        Err(_) => return S5Error::new(ErrorKind::Input, "Descriptor").c_stringify(),
    };

    let node_address_cstr = CStr::from_ptr(node_address);
    let node_address: &str = match node_address_cstr.to_str() {
        Ok(string) => {
            if string.contains("electrum") || string.contains("http") {
                string
            } else {
                DEFAULT
            }
        }
        Err(_) => DEFAULT,
    };

    let config = match WalletConfig::new(descriptor, node_address, None) {
        Ok(conf) => conf,
        Err(e) => return S5Error::new(ErrorKind::Internal, &e.message).c_stringify(),
    };

    let to_address_cstr = CStr::from_ptr(to_address);
    let to_address: &str = match to_address_cstr.to_str() {
        Ok(string) => string,
        Err(_) => return S5Error::new(ErrorKind::Input, "To-Address").c_stringify(),
    };

    let sweep_cstr = CStr::from_ptr(sweep);
    let sweep: bool = match sweep_cstr.to_str() {
        Ok(string) => string == "true",
        Err(_) => false,
    };

    let amount_cstr = CStr::from_ptr(amount);
    let amount: Option<u64> = match amount_cstr.to_str() {
        Ok(string) => match string.parse::<u64>() {
            Ok(i) => {
                if sweep {
                    None
                } else {
                    Some(i)
                }
            }
            Err(_) => return S5Error::new(ErrorKind::Input, "Invalid Amount.").c_stringify(),
        },
        Err(_) => return S5Error::new(ErrorKind::Input, "Amount").c_stringify(),
    };

    let fee_absolute_cstr = CStr::from_ptr(fee_absolute);
    let fee_absolute: u64 = match fee_absolute_cstr.to_str() {
        Ok(string) => match string.parse::<u64>() {
            Ok(i) => i,
            Err(_) => return S5Error::new(ErrorKind::Input, "Fee Rate").c_stringify(),
        },
        Err(_) => return S5Error::new(ErrorKind::Input, "Fee Rate").c_stringify(),
    };

    match psbt::build(config, to_address, amount, fee_absolute, sweep) {
        Ok(result) => result.c_stringify(),
        Err(e) => e.c_stringify(),
    }
}

/// Decodes a PSBT and returns all outputs of the transaction and total size.
/// "miner" is used in the 'to' field of an output to indicate fee.
/// - *OUTPUT*
/// ```
///   DecodedTx{
///     outputs: Vec<DecodedTxIO {
///       value: u64,
///       to: String,
///     }>
///   }
/// ```
/// # Safety
/// - This function is unsafe because it dereferences and a returns raw pointer.
/// - ENSURE that result is passed into cstring_free(ptr: *mut c_char) after use.
#[no_mangle]
pub unsafe extern "C" fn decode_psbt(network: *const c_char, psbt: *const c_char) -> *mut c_char {
    let network_cstr = CStr::from_ptr(network);
    let network_str: &str = match network_cstr.to_str() {
        Ok(string) => string,
        Err(_) => "test",
    };
    let network = match network_str {
        "main" => Network::Bitcoin,
        "test" => Network::Testnet,
        _ => Network::Testnet,
    };

    let psbt_cstr = CStr::from_ptr(psbt);
    let psbt: &str = match psbt_cstr.to_str() {
        Ok(string) => string,
        Err(_) => return S5Error::new(ErrorKind::Input, "PSBT-Input").c_stringify(),
    };

    match psbt::decode(network, psbt) {
        Ok(result) => result.c_stringify(),
        Err(e) => e.c_stringify(),
    }
}

/// Signs a PSBT with a descriptor.
/// Can only be used with descriptors containing private key(s).
/// - *OUTPUT*
/// ```
///  WalletPSBT {
///    pub psbt: String,
///    pub is_finalized: bool,
///  }
/// ```
/// # Safety
/// - This function is unsafe because it dereferences and a returns raw pointer.
/// - ENSURE that result is passed into cstring_free(ptr: *mut c_char) after use.
#[no_mangle]
pub unsafe extern "C" fn sign_tx(
    descriptor: *const c_char,
    unsigned_psbt: *const c_char,
) -> *mut c_char {
    let descriptor_cstr = CStr::from_ptr(descriptor);
    let descriptor: &str = match descriptor_cstr.to_str() {
        Ok(string) => string,
        Err(_) => return S5Error::new(ErrorKind::Input, "Descriptor").c_stringify(),
    };

    let config = match WalletConfig::new_offline(descriptor) {
        Ok(conf) => conf,
        Err(e) => return S5Error::new(ErrorKind::Internal, &e.message).c_stringify(),
    };

    let unsigned_psbt_cstr = CStr::from_ptr(unsigned_psbt);
    let unsigned_psbt: &str = match unsigned_psbt_cstr.to_str() {
        Ok(string) => string,
        Err(_) => return S5Error::new(ErrorKind::Input, "Descriptor").c_stringify(),
    };

    match psbt::sign(config, unsigned_psbt) {
        Ok(result) => result.c_stringify(),
        Err(e) => e.c_stringify(),
    }
}

/// Broadcasts a signed transaction to a remote node.
/// - *OUTPUT*
/// ```
///  Txid {
///    pub txid: String,
///  }
/// ```
/// # Safety
/// - This function is unsafe because it dereferences and a returns raw pointer.
/// - ENSURE that result is passed into cstring_free(ptr: *mut c_char) after use.
#[no_mangle]
pub unsafe extern "C" fn broadcast_tx(
    descriptor: *const c_char,
    node_address: *const c_char,
    signed_psbt: *const c_char,
) -> *mut c_char {
    let descriptor_cstr = CStr::from_ptr(descriptor);
    let descriptor: &str = match descriptor_cstr.to_str() {
        Ok(string) => string,
        Err(_) => return S5Error::new(ErrorKind::Input, "Descriptor").c_stringify(),
    };

    let node_address_cstr = CStr::from_ptr(node_address);
    let node_address: &str = match node_address_cstr.to_str() {
        Ok(string) => {
            if string.contains("electrum") || string.contains("http") {
                string
            } else {
                DEFAULT
            }
        }
        Err(_) => DEFAULT,
    };

    let config = match WalletConfig::new(descriptor, node_address, None) {
        Ok(conf) => conf,
        Err(e) => return e.c_stringify(),
    };

    let psbt_cstr = CStr::from_ptr(signed_psbt);
    let signed_psbt: &str = match psbt_cstr.to_str() {
        Ok(string) => string,
        Err(_) => return S5Error::new(ErrorKind::Input, "Descriptor").c_stringify(),
    };

    match psbt::broadcast(config, signed_psbt) {
        Ok(result) => result.c_stringify(),
        Err(e) => e.c_stringify(),
    }
}

/// Checks if an extended public key is valid.
/// Do not use the key source while checking an xpub i.e. remove [fingerprint/derivation/path/values] and only provide the xpub/tpub.
/// - *OUTPUT*
/// ```
/// "true" | "false"
/// ```
/// # Safety
/// - This function is unsafe because it dereferences and a returns raw pointer.
/// - ENSURE that result is passed into cstring_free(ptr: *mut c_char) after use.
#[no_mangle]
pub unsafe extern "C" fn check_xpub(xpub: *const c_char) -> *mut c_char {
    let xpub_cstr = CStr::from_ptr(xpub);
    let xpub: &str = match xpub_cstr.to_str() {
        Ok(string) => string,
        Err(_) => return CString::new("false").unwrap().into_raw(),
    };

    match derivation::check_xpub(xpub) {
        true => CString::new("true").unwrap().into_raw(),
        false => CString::new("false").unwrap().into_raw(),
    }
}

/// Gets the current block height.
/// - *OUTPUT*
/// ```  
///  BlockHeight {
///    height: u32,
///  }
/// ```
/// # Safety
/// - This function is unsafe because it dereferences and a returns raw pointer.
/// - ENSURE that result is passed into cstring_free(ptr: *mut c_char) after use.
#[no_mangle]
pub unsafe extern "C" fn get_height(
    network: *const c_char,
    node_address: *const c_char,
) -> *mut c_char {
    let network_cstr = CStr::from_ptr(network);
    let network: &str = match network_cstr.to_str() {
        Ok(string) => string,
        Err(_) => "test",
    };
    let network_enum = match network {
        "main" => Network::Bitcoin,
        _ => Network::Testnet,
    };
    let node_address_cstr = CStr::from_ptr(node_address);
    let node_address: &str = match node_address_cstr.to_str() {
        Ok(string) => {
            if string == DEFAULT {
                match network_enum {
                    Network::Bitcoin => DEFAULT_MAINNET_NODE,
                    _ => DEFAULT_TESTNET_NODE,
                }
            } else {
                string
            }
        }
        Err(_) => match network_enum {
            Network::Bitcoin => DEFAULT_MAINNET_NODE,
            _ => DEFAULT_TESTNET_NODE,
        },
    };

    let config = match WalletConfig::new("*", node_address, None) {
        Ok(conf) => conf,
        Err(e) => return S5Error::new(ErrorKind::Internal, &e.message).c_stringify(),
    };
    match height::get_height(config) {
        Ok(result) => result.c_stringify(),
        Err(e) => e.c_stringify(),
    }
}

/// Convert days to bitcoin blocks as height. *Does not consider the current block height*. Add to get_height to get the expected block height after the given time in days.
/// - *OUTPUT*
/// ```  
///  BlockHeight {
///    height: u32,
///  }
/// ```
/// # Safety
/// - This function is unsafe because it dereferences and a returns raw pointer.
/// - ENSURE that result is passed into cstring_free(ptr: *mut c_char) after use.
#[no_mangle]
pub unsafe extern "C" fn days_to_blocks(days: *const c_char) -> *mut c_char {
    let days_cstr = CStr::from_ptr(days);
    let days: u32 = match days_cstr.to_str() {
        Ok(string) => match string.parse::<u32>() {
            Ok(i) => i,
            Err(_) => {
                return S5Error::new(ErrorKind::Input, "Days must be a stringified uint32")
                    .c_stringify()
            }
        },
        Err(_) => return S5Error::new(ErrorKind::Input, "Days").c_stringify(),
    };

    height::BlockHeight { height: days * 144 }.c_stringify()
}

/// Switch on tor daemon.
/// BETA: Careful with this.
/// # Safety
/// - This function is unsafe because it dereferences and a returns raw pointer.
/// - ENSURE that result is passed into cstring_free(ptr: *mut c_char) after use.
#[no_mangle]
pub unsafe extern "C" fn tor_start(tmp_path: *mut c_char) -> *mut c_char {
    let tmp_path_cstr = CStr::from_ptr(tmp_path);
    let tmp_path: &str = match tmp_path_cstr.to_str() {
        Ok(string) => string,
        Err(_) => "/tmp",
    };

    let control_key = tor::start(tmp_path);
    CString::new(control_key).unwrap().into_raw()
}

/// Get bootstrap progress from tor daemon. Wait ~1s after calling tor_start() before calling this.
/// BETA: Careful with this.
/// # Safety
/// - This function is unsafe because it dereferences and a returns raw pointer.
/// - ENSURE that result is passed into cstring_free(ptr: *mut c_char) after use.
#[no_mangle]
pub unsafe extern "C" fn tor_progress(control_key: *mut c_char) -> *mut c_char {
    let control_key_cstr = CStr::from_ptr(control_key);
    let control_key: &str = match control_key_cstr.to_str() {
        Ok(string) => string,
        Err(_) => return S5Error::new(ErrorKind::Input, "Control-Key").c_stringify(),
    };
    match tor::bootstrap_progress(control_key) {
        Ok(result) => CString::new(result.to_string()).unwrap().into_raw(),
        Err(e) => e.c_stringify(),
    }
}

/// Shutdown tor daemon.
/// BETA: Careful with this.
/// # Safety
/// - This function is unsafe because it dereferences and a returns raw pointer.
/// - ENSURE that result is passed into cstring_free(ptr: *mut c_char) after use.
#[no_mangle]
pub unsafe extern "C" fn tor_stop(control_key: *mut c_char) -> *mut c_char {
    let control_key_cstr = CStr::from_ptr(control_key);
    let control_key: &str = match control_key_cstr.to_str() {
        Ok(string) => string,
        Err(_) => return S5Error::new(ErrorKind::Input, "Control-Key").c_stringify(),
    };

    match tor::shutdown(control_key) {
        Ok(result) => CString::new(result.to_string()).unwrap().into_raw(),
        Err(e) => e.c_stringify(),
    }
}

/// After using any other function, pass the output pointer into cstring_free(ptr: *mut c_char) to clear memory.
/// ALWAYS use this in combination with any other function.
/// Failure to do so can lead to memory bugs.
/// # Safety
/// - This function is unsafe because it deferences a raw pointer.
#[no_mangle]
pub unsafe extern "C" fn cstring_free(ptr: *mut c_char) {
    if ptr.is_null() {
        return;
    }
    CString::from_raw(ptr);
    // rust automatically deallocates the pointer after using it
    // here we just convert it to a CString so it is used and cleared
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    /// ENSURE that mnemonic does not error for bad input values.
    /// Default to 24 words mnemonic.
    fn test_ffi_c_master_ops() {
        unsafe {
            let master = generate_master(
                CString::new("notanumber").unwrap().into_raw(),
                CString::new("9").unwrap().into_raw(),
                CString::new("").unwrap().into_raw(),
            );
            // unrecognized network string must default to test
            //length 9 should default to 24 words
            let master = CStr::from_ptr(master).to_str().unwrap();
            let master: seed::MasterKey = serde_json::from_str(master).unwrap();
            assert_eq!(
                24,
                master
                    .mnemonic
                    .split_whitespace()
                    .collect::<Vec<&str>>()
                    .len()
            );

            let mnemonic = "panel across strong judge economy song loud valid regret fork consider bid rack young avoid soap plate injury snow crater beef alone stay clock";
            let fingerprint = "eb79e0ff";
            let xprv = "tprv8ZgxMBicQKsPduTkddZgfGyk4ZJjtEEZQjofpyJg74LizJ469DzoF8nmU1YcvBFskXVKdoYmLoRuZZR1wuTeuAf8rNYR2zb1RvFns2Vs8hY";
            let master = import_master(
                CString::new("notanumber").unwrap().into_raw(),
                CString::new(mnemonic).unwrap().into_raw(),
                CString::new("").unwrap().into_raw(),
            );
            let master = CStr::from_ptr(master).to_str().unwrap();
            let master: seed::MasterKey = serde_json::from_str(master).unwrap();
            assert_eq!(xprv, master.xprv);
            assert_eq!(fingerprint, master.fingerprint);
        }
    }
    //     /**
    //      * MasterKey {
    //         mnemonic: "panel across strong judge economy song loud valid regret fork consider bid rack young avoid soap plate injury snow crater beef alone stay clock",
    //         fingerprint: "eb79e0ff",
    //         xprv: "tprv8ZgxMBicQKsPduTkddZgfGyk4ZJjtEEZQjofpyJg74LizJ469DzoF8nmU1YcvBFskXVKdoYmLoRuZZR1wuTeuAf8rNYR2zb1RvFns2Vs8hY",
    //     }
    //      */
    #[test]
    fn test_ffi_child_ops() {
        unsafe {
            let fingerprint = "eb79e0ff";
            let master_xprv: &str = "tprv8ZgxMBicQKsPduTkddZgfGyk4ZJjtEEZQjofpyJg74LizJ469DzoF8nmU1YcvBFskXVKdoYmLoRuZZR1wuTeuAf8rNYR2zb1RvFns2Vs8hY";
            let master_xprv_cstr = CString::new(master_xprv).unwrap().into_raw();

            let purpose_index = "84";
            let purpose_cstr = CString::new(purpose_index).unwrap().into_raw();

            let account_index = "0";
            let account_cstr = CString::new(account_index).unwrap().into_raw();
            let hardened_path = "m/84h/1h/0h";
            let account_xprv = "tprv8gqqcZU4CTQ9bFmmtVCfzeSU9ch3SfgpmHUPzFP5ktqYpnjAKL9wQK5vx89n7tgkz6Am42rFZLS9Qs4DmFvZmgukRE2b5CTwiCWrJsFUoxz";
            let account_xpub = "tpubDDXskyWJLq5pUioZn8sGQ46aieCybzsjLb5BGmRPBAdwfGyvwiyXaoho8EYJcgJa5QGHGYpDjLQ8gWzczWbxadeRkCuExW32Boh696yuQ9m";
            let child_keys = derivation::ChildKeys {
                fingerprint: fingerprint.to_string(),
                hardened_path: hardened_path.to_string(),
                xprv: account_xprv.to_string(),
                xpub: account_xpub.to_string(),
                
            };

            let stringified = serde_json::to_string(&child_keys).unwrap();

            let result = derive_hardened(master_xprv_cstr, purpose_cstr, account_cstr);
            let result_cstr = CStr::from_ptr(result);
            let result: &str = result_cstr.to_str().unwrap();
            assert_eq!(result, stringified);
        }
    }

    #[test]
    fn test_ffi_wallet() {
        unsafe {
            let xkey = "[db7d25b5/84'/1'/6']tpubDCCh4SuT3pSAQ1qAN86qKEzsLoBeiugoGGQeibmieRUKv8z6fCTTmEXsb9yeueBkUWjGVzJr91bCzeCNShorbBqjZV4WRGjz3CrJsCboXUe";
            let node_address_cstr = CString::new("default").unwrap().into_raw();

            let descriptor = format!("wsh(pk({}/*))", xkey);
            let descriptor_cstr = CString::new(descriptor).unwrap().into_raw();
            let balance_ptr = sync_balance(descriptor_cstr, node_address_cstr);
            let balance_str = CStr::from_ptr(balance_ptr).to_str().unwrap();
            let balance: history::WalletBalance = serde_json::from_str(balance_str).unwrap();
            assert_eq!(balance.balance, 10_000);
            let index_cstr = CString::new("0").unwrap().into_raw();
            let address_ptr = get_address(descriptor_cstr, index_cstr);
            let address_str = CStr::from_ptr(address_ptr).to_str().unwrap();
            let address: address::WalletAddress = serde_json::from_str(address_str).unwrap();
            assert_eq!(
                address.address,
                "tb1q5f3jl5lzlxtmhptfe9crhmv4wh392ku5ztkpt6xxmqqx2c3jyxrs8vgat7"
            );
            let network_cstr = CString::new("test").unwrap().into_raw();

            let conf_target = CString::new("1").unwrap().into_raw();
            let fees = estimate_network_fee(network_cstr, node_address_cstr, conf_target);
            let fees_str = CStr::from_ptr(fees).to_str().unwrap();

            let fees_struct: fees::NetworkFee = serde_json::from_str(fees_str).unwrap();
            assert!(fees_struct.rate >= 1.0);
        }
    }
    #[test]
    fn test_ffi_history() {
        unsafe {
            let descriptor = "wpkh([71b57c5d/84h/1h/0h]tprv8fUHbn7Tng83h8SvS6JLXM2bTViJai8N31obfNxAyXzaPxiyCxFqxeewBbcDu8jvpbquTW3577nRJc1KLChurPs6rQRefWTgUFH1ZnjU2ap/*)";
            let descriptor_cstr = CString::new(descriptor).unwrap().into_raw();
            let node_address_cstr = CString::new("default").unwrap().into_raw();
            let history_ptr = sync_history(descriptor_cstr, node_address_cstr);
            let history_str = CStr::from_ptr(history_ptr).to_str().unwrap();
            let history: history::WalletHistory = serde_json::from_str(history_str).unwrap();
            println!("{:#?}", history);
            // assert_eq!(history.history.len(),3);
        }
    }

    #[test]
    fn test_days_to_blocks() {
        unsafe {
            let one_day = "1";
            let blocks_ptr = days_to_blocks(CString::new(one_day).unwrap().into_raw());
            let blocks_str = CStr::from_ptr(blocks_ptr).to_str().unwrap();
            let blocks: height::BlockHeight = serde_json::from_str(blocks_str).unwrap();
            assert_eq!(blocks.height, 144);
            let one_year_as_days = "365";
            let blocks_ptr = days_to_blocks(CString::new(one_year_as_days).unwrap().into_raw());
            let blocks_str = CStr::from_ptr(blocks_ptr).to_str().unwrap();
            let blocks: height::BlockHeight = serde_json::from_str(blocks_str).unwrap();
            assert_eq!(blocks.height, 52560);
        }
    }
}
