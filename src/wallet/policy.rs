use std::ffi::CString;
use std::os::raw::c_char;
use std::str::FromStr;

use serde::{Deserialize, Serialize};

use bdk::descriptor::Segwitv0;
use bdk::descriptor::{Descriptor, Legacy, Miniscript};
use bdk::miniscript::policy::Concrete;
// use bdk::Error;
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

pub fn compile(
  policy: &str, 
  script_type: &str
) -> Result<WalletPolicy, S5Error> {
  let x_policy = match Concrete::<String>::from_str(policy) {
    Ok(result) => result,
    Err(_) => return Err(S5Error::new(ErrorKind::Input, "Invalid Policy")),
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

#[cfg(test)]
mod tests {
  use super::*;
  use crate::config::{WalletConfig, DEFAULT_TESTNET_NODE};
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
      "or(pk({}),and(pk({}),after({})))",
      user_xprv, custodian, bailout_time
    );

    //  DESCRIPTORS
    let raft_result_bech32 = compile(&raft_policy, "wsh").unwrap();
    let expected_raft_wsh = "wsh(or_d(pk([db7d25b5/84'/1'/6']tprv8fWev2sCuSkVWYoNUUSEuqLkmmfiZaVtgxosS5jRE9fw5ejL2odsajv1QyiLrPri3ppgyta6dsFaoDVCF4ZdEAR6qqY4tnaosujsPzLxB49/0/*),and_v(v:pk([66a0c105/84'/1'/5']tpubDCKvnVh6U56wTSUEJGamQzdb3ByAc6gTPbjxXQqts5Bf1dBMopknipUUSmAV3UuihKPTddruSZCiqhyiYyhFWhz62SAGuC3PYmtAafUuG6R/0/*),after(595600))))";

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
      WalletConfig::default(expected_raft_wsh, DEFAULT_TESTNET_NODE).unwrap();
    let single_config: WalletConfig =
      WalletConfig::default(expected_single_wpkh, DEFAULT_TESTNET_NODE).unwrap();
    let watchonly_config: WalletConfig =
      WalletConfig::default(expected_single_watchonly_wpkh, DEFAULT_TESTNET_NODE).unwrap();

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
}
