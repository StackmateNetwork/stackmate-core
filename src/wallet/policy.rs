use std::collections::btree_map::BTreeMap;
use std::ffi::CString;
use std::os::raw::c_char;
use std::str::FromStr;

use serde::{Deserialize, Serialize};

use bdk::descriptor::policy::Policy;
use bdk::descriptor::{Descriptor, Legacy, Miniscript, Segwitv0};
use bdk::miniscript::policy::Concrete;

use bdk::descriptor::policy::SatisfiableItem;

use bdk::blockchain::noop_progress;
use bdk::database::MemoryDatabase;

use bdk::{KeychainKind, Wallet};
// use bdk::Error;
use crate::config::WalletConfig;
use crate::e::{ErrorKind, S5Error};

/// FFI Output
#[derive(Serialize, Deserialize, Debug)]
pub struct WalletPolicy {
  pub policy: String,
  pub descriptor: String,
}
impl WalletPolicy {
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

pub fn compile(policy: &str, script_type: &str) -> Result<WalletPolicy, S5Error> {
  let x_policy = match Concrete::<String>::from_str(policy) {
    Ok(result) => result,
    Err(e) => {
      eprintln!("{:#?}", e.to_string());
      return Err(S5Error::new(ErrorKind::Input, "Invalid Policy"));
    }
  };

  let legacy_policy: Miniscript<String, Legacy> = match x_policy.compile() {
    Ok(result) => result,
    Err(e) => return Err(S5Error::new(ErrorKind::Internal, &e.to_string())),
  };
  // .map_err(|e| Error::Generic(e.to_string())).unwrap();
  let segwit_policy: Miniscript<String, Segwitv0> = match x_policy.compile() {
    Ok(result) => result,
    Err(e) => return Err(S5Error::new(ErrorKind::Internal, &e.to_string())),
  };

  let descriptor = match script_type {
    "wpkh" => policy.replace("pk", "wpkh"),
    "sh" => Descriptor::new_sh(legacy_policy).unwrap().to_string(),
    "wsh" => Descriptor::new_wsh(segwit_policy).unwrap().to_string(),
    "sh-wsh" => Descriptor::new_sh_wsh(segwit_policy).unwrap().to_string(),
    _ => return Err(S5Error::new(ErrorKind::Internal, "Invalid-Script-Type")),
  };

  Ok(WalletPolicy {
    policy: policy.to_string(),
    descriptor: descriptor.split('#').collect::<Vec<&str>>()[0].to_string(),
  })
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SpendingPolicyPaths {
  pub internal: BTreeMap<String, Vec<usize>>,
  pub external: BTreeMap<String, Vec<usize>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RaftMemberPolicyPaths {
  pub primary: SpendingPolicyPaths,
  pub secondary: SpendingPolicyPaths,
}

pub fn raft_policy_paths(config: WalletConfig) -> Result<RaftMemberPolicyPaths, S5Error> {
  let wallet = match Wallet::new(
    &config.deposit_desc,
    Some(&config.change_desc),
    config.network,
    MemoryDatabase::default(),
    config.client,
  ) {
    Ok(result) => result,
    Err(_) => return Err(S5Error::new(ErrorKind::Internal, "Wallet-Initialization")),
  };

  match wallet.sync(noop_progress(), None) {
    Ok(_) => (),
    Err(_) => return Err(S5Error::new(ErrorKind::Internal, "Wallet-Sync")),
  };

  let ext_policies = match wallet.policies(KeychainKind::External) {
    Ok(result) => result,
    Err(_) => return Err(S5Error::new(ErrorKind::Internal, "Wallet-Policy")),
  };

  let int_policies = match wallet.policies(KeychainKind::Internal) {
    Ok(result) => result,
    Err(_) => return Err(S5Error::new(ErrorKind::Internal, "Wallet-Policy")),
  };

  let mut primary_ext_path = BTreeMap::new();
  primary_ext_path.insert(ext_policies.clone().unwrap().item.id().to_string(), vec![0]);


  let mut primary_int_path = BTreeMap::new();
  primary_int_path.insert(int_policies.clone().unwrap().item.id().to_string(), vec![0]);


  let mut secondary_ext_path = BTreeMap::new();
  secondary_ext_path.insert(ext_policies.clone().unwrap().item.id().to_string(), vec![1]);


  let mut secondary_int_path = BTreeMap::new();
  secondary_int_path.insert(int_policies.clone().unwrap().item.id().to_string(), vec![1]);

  // match ext_policies.clone().unwrap().item{
  //   SatisfiableItem::Thresh { items, threshold } => {
  //     primary_ext_path.insert(items[0].id.to_string(), vec![0,1]);
  //     secondary_ext_path.insert(items[1].id.to_string(), vec![0,1]);

  //     match &items[1].item{
  //       SatisfiableItem::Thresh { items, threshold } => {
  //         secondary_ext_path.insert(items[0].id.to_string(), vec![2]);
  //         // println!("SECONDARY{:#?}", items);
  //       },
  //       _ => println!("{}", "Not-Thresh"),

  //     }
  //   }
  //   _ => println!("{}", "Not-Thresh"),
  // };

  // match int_policies.clone().unwrap().item{
  //   SatisfiableItem::Thresh { items, threshold } => {
  //     primary_int_path.insert(items[0].id.to_string(), vec![1]);
  //     secondary_int_path.insert(items[1].id.to_string(), vec![0,1]);

  //     match &items[1].item{
  //       SatisfiableItem::Thresh { items, threshold } => {
  //         secondary_int_path.insert(items[0].id.to_string(), vec![0]);
  //         // println!("SECONDARY{:#?}", items);
  //       },
  //       _ => println!("{}", "Not-Thresh"),

  //     }
  //   }
  //   _ => println!("{}", "Not-Thresh"),
  // };

  // println!("{:#?}", ext_policies.unwrap().item);

  // FIXME Find correct spending policy path for secondary
  Ok(RaftMemberPolicyPaths {
    primary: SpendingPolicyPaths {
      internal: primary_int_path.clone(),
      external: primary_ext_path.clone(),
    },
    secondary: SpendingPolicyPaths {
      internal: secondary_int_path,
      external: secondary_ext_path,
    },
  })
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::config::{WalletConfig, DEFAULT_TESTNET_NODE, BlockchainBackend};
  use crate::wallet::address::generate;

  #[test]
  fn test_policies() {
    let user_xprv = "[db7d25b5/84'/1'/6']tprv8fWev2sCuSkVWYoNUUSEuqLkmmfiZaVtgxosS5jRE9fw5ejL2odsajv1QyiLrPri3ppgyta6dsFaoDVCF4ZdEAR6qqY4tnaosujsPzLxB49/0/*";
    let user_xpub = "[db7d25b5/84'/1'/6']tpubDCCh4SuT3pSAQ1qAN86qKEzsLoBeiugoGGQeibmieRUKv8z6fCTTmEXsb9yeueBkUWjGVzJr91bCzeCNShorbBqjZV4WRGjz3CrJsCboXUe/0/*";
    let custodian = "[66a0c105/84'/1'/5']tpubDCKvnVh6U56wTSUEJGamQzdb3ByAc6gTPbjxXQqts5Bf1dBMopknipUUSmAV3UuihKPTddruSZCiqhyiYyhFWhz62SAGuC3PYmtAafUuG6R/0/*";
    let bailout_time = 595_600;
    // POLICIES
    let single_policy = format!("pk({})", user_xprv);
    let single_watchonly_policy = format!("pk({})", user_xpub);
    let raft_policy = format!(
      "thresh(1,pk({}),and(pk({}),after({})))",
      user_xprv, custodian, bailout_time
    );
    // let raft_policy_rest = format!(
    //   "or(pk({}),and(pk({}),after({})))",
    //   user_xprv, custodian, bailout_time
    // );

    // println!("{}", raft_policy_rest);

    //  DESCRIPTORS
    let raft_result_bech32 = compile(&raft_policy, "wsh").unwrap();
    // let expected_raft_wsh = "wsh(or_d(pk([db7d25b5/84'/1'/6']tprv8fWev2sCuSkVWYoNUUSEuqLkmmfiZaVtgxosS5jRE9fw5ejL2odsajv1QyiLrPri3ppgyta6dsFaoDVCF4ZdEAR6qqY4tnaosujsPzLxB49/0/*),and_v(v:pk([66a0c105/84'/1'/5']tpubDCKvnVh6U56wTSUEJGamQzdb3ByAc6gTPbjxXQqts5Bf1dBMopknipUUSmAV3UuihKPTddruSZCiqhyiYyhFWhz62SAGuC3PYmtAafUuG6R/0/*),after(595600))))";

    let expected_raft_wsh = "wsh(thresh(1,pk([db7d25b5/84'/1'/6']tprv8fWev2sCuSkVWYoNUUSEuqLkmmfiZaVtgxosS5jRE9fw5ejL2odsajv1QyiLrPri3ppgyta6dsFaoDVCF4ZdEAR6qqY4tnaosujsPzLxB49/0/*),snj:and_v(v:pk([66a0c105/84'/1'/5']tpubDCKvnVh6U56wTSUEJGamQzdb3ByAc6gTPbjxXQqts5Bf1dBMopknipUUSmAV3UuihKPTddruSZCiqhyiYyhFWhz62SAGuC3PYmtAafUuG6R/0/*),after(595600))))";
    let single_result_bech32 = compile(&single_policy, "wpkh").unwrap();
    println!("{:#?}", single_result_bech32);

    let expected_single_wpkh = "wpkh([db7d25b5/84'/1'/6']tprv8fWev2sCuSkVWYoNUUSEuqLkmmfiZaVtgxosS5jRE9fw5ejL2odsajv1QyiLrPri3ppgyta6dsFaoDVCF4ZdEAR6qqY4tnaosujsPzLxB49/0/*)";

    let single_watchonly_result_bech32 = compile(&single_watchonly_policy, "wpkh").unwrap();
    let expected_single_watchonly_wpkh = "wpkh([db7d25b5/84'/1'/6']tpubDCCh4SuT3pSAQ1qAN86qKEzsLoBeiugoGGQeibmieRUKv8z6fCTTmEXsb9yeueBkUWjGVzJr91bCzeCNShorbBqjZV4WRGjz3CrJsCboXUe/0/*)";

    assert_eq!(&raft_result_bech32.descriptor, expected_raft_wsh);
    assert_eq!(&single_result_bech32.descriptor, expected_single_wpkh);
    assert_eq!(
      &single_watchonly_result_bech32.descriptor,
      expected_single_watchonly_wpkh
    );

    // let raft_result_p2sh = compile(&raft_policy, "sh").unwrap();
    // let single_result_p2sh = compile(&single_policy, "sh").unwrap();
    // let single_watchonly_result_p2sh = compile(&single_watchonly_policy, "sh").unwrap();

    // let raft_result_legacy = compile(&raft_policy, "pk").unwrap();
    // let single_result_legacy = compile(&single_policy, "pk").unwrap();
    // let single_watchonly_result_legacy = compile(&single_watchonly_policy, "pk").unwrap();

    let raft_config: WalletConfig =
      WalletConfig::new(expected_raft_wsh, BlockchainBackend::Electrum, DEFAULT_TESTNET_NODE, None).unwrap();
    let single_config: WalletConfig =
      WalletConfig::new(expected_single_wpkh, BlockchainBackend::Electrum, DEFAULT_TESTNET_NODE, None).unwrap();
    let watchonly_config: WalletConfig =
      WalletConfig::new(expected_single_watchonly_wpkh, BlockchainBackend::Electrum, DEFAULT_TESTNET_NODE, None).unwrap();

    let raft_bech32_address = generate(raft_config, 0);
    let single_bech32_address = generate(single_config, 0);
    let watchonly_bech32_address = generate(watchonly_config, 0);

    println!("{:?}", raft_bech32_address);
    println!("{:?}", single_bech32_address);
    println!("{:?}", watchonly_bech32_address);
  }

  use bdk::keys::{DescriptorKey, ExtendedKey};

  use bdk::descriptor;
  use bdk::keys::DerivableKey;
  use bitcoin::util::bip32::DerivationPath;
  use bitcoin::util::bip32::ExtendedPubKey;
  use bitcoin::util::bip32::Fingerprint;

  #[test]
  fn test_bare_wpkh_desc() {
    let user_xpub = "tpubDCCh4SuT3pSAQ1qAN86qKEzsLoBeiugoGGQeibmieRUKv8z6fCTTmEXsb9yeueBkUWjGVzJr91bCzeCNShorbBqjZV4WRGjz3CrJsCboXUe";
    let xpub = ExtendedPubKey::from_str(user_xpub).unwrap();
    let fingerprint = Fingerprint::from_str("db7d25b5").unwrap();
    let hardened_path = DerivationPath::from_str("m/84'/1'/6'").unwrap();
    let unhardened_path = DerivationPath::from_str("m/0").unwrap();

    let exkey: ExtendedKey<Segwitv0> = ExtendedKey::from(xpub);

    let dkey: DescriptorKey<Segwitv0> = exkey
      .into_descriptor_key(Some((fingerprint, hardened_path)), unhardened_path)
      .unwrap();

    // println!("{:#?}",dkey);

    // let policy = bdk::fragment!(pk(dkey)).unwrap();
    // println!("{:#?}",policy);

    let (desc, _, _) = descriptor! {wpkh(dkey)}.unwrap();
    println!("{:#?}", desc.to_string());
    // println!("{:#?}",key_map);
    // println!("{:#?}",networks);
  }

  #[test]
  fn test_raft() {
    let policy = "or(pk([f128c8df/84h/1h/0h]tprv8fM5yWPWNuAU8wnYSVJed4xqGX5G9XEZHsMoy1wydWecBthUiJFDoKGqtAYZ2K9m1cfPSJvpGRyqgm8pdPWmGuj1nh8vTiuwEQdvPfDLS72/0/*),and(pk([05232dee/84h/1h/0h]tpubDCLDXhTEBD9usoa7td6k94WhnA8G8gLPnEkZeauvTqyB2NgV9hZkVbWeQmmSbDxYWuvcsiqg2DY688NiXzjZwt3TZAxYs33RDXvpqPNSdPM/0/*),after(2110534)))";
    let result = compile(&policy, "wsh").unwrap();
    println!("{:#?}", result);
    let raft_config = WalletConfig::new(&result.descriptor, BlockchainBackend::Electrum, DEFAULT_TESTNET_NODE, None).unwrap();
    let spending_policies = raft_policy_paths(raft_config).unwrap();
    println!("{:#?}", spending_policies);
  }
}
