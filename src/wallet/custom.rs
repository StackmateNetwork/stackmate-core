use std::fmt::Debug;
use std::fmt::Display;
use std::fmt::Formatter;

#[derive(Debug)]
pub enum CustomWallet {
  Solo,
  Escrow,
  Team,
  Raft,
  Custom,
}
impl Display for CustomWallet {
  fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
    match self {
      CustomWallet::Solo => write!(f, "Solo"),
      CustomWallet::Escrow => write!(f, "Escrow"),
      CustomWallet::Team => write!(f, "Team"),
      CustomWallet::Raft => write!(f, "Raft"),
      CustomWallet::Custom => write!(f, "Custom"),
    }
  }
}

#[cfg(test)]
mod tests {
  use std::collections::BTreeMap;
  use crate::config::{WalletConfig,DEFAULT_TESTNET_NODE};
  use crate::key::{
    derivation::{ChildKeys, DerivationPurpose},
    seed::MasterKey,
  };
  use crate::wallet::{policy, address, history, psbt};

  #[test]
  fn test_escrow_wallet_offline() {
    let a_master = MasterKey{
      fingerprint: "c2cb6b81".to_string(),
      mnemonic: "since tiger lend garment usage eyebrow drive staff grant force front swing medal scout side index kitchen marriage hockey habit smooth bread network giant".to_string(),
      xprv: "tprv8ZgxMBicQKsPejWK14KjNayPfWwudPKwfDx2YYB2XqSbPKWobGoicV4uYMt3aLHBXFcN3Zkf2rK2Nmy1XCTtH8VDtxAoy4GtRJphEnAzx6n".to_string()
    };
    let a_child =
      ChildKeys::from_hardened_account(&a_master.xprv, DerivationPurpose::Native, 0).unwrap();

    let b_master = MasterKey{
      fingerprint: "9eee95d3".to_string(),
      mnemonic: "flower increase image street chair wagon comic volume beyond vote tenant machine math modify rival example salute distance strategy trim joy gossip close remain".to_string(),
      xprv: "tprv8ZgxMBicQKsPd882YUvsSKW1f26NvXjtE4CyMbEotRmjsDJHAh9s2HHg7QjQW6SNo1xuoBRsFH68LcyPt5hJCCuUxQobpmpSu32AVrTf8JD".to_string()
    };
    let b_child =
      ChildKeys::from_hardened_account(&b_master.xprv, DerivationPurpose::Native, 0).unwrap();

    let e_master = MasterKey{
      fingerprint: "958b4ad7".to_string(),
      mnemonic: "digital middle letter fury rebuild conduct bronze radio mansion dinner assume world mind utility divert obscure fit entry cat soda mix breeze orchard document".to_string(),
      xprv: "tprv8ZgxMBicQKsPfBANiurYC1Jm3G9WmP8ZFq5QhyPASTBWuNo1sdS518negGTeD7ARdsQmob9WbKk6hNqHN4torUcCq7SpDpVfaVG7PefbmGe".to_string()
    };
    let e_child =
      ChildKeys::from_hardened_account(&e_master.xprv, DerivationPurpose::Native, 0).unwrap();

    let escrow_public_policy = format!(
      "thresh(2,pk({}),pk({}),pk({}))",
      a_child.to_extended_xpub_str(),
      b_child.to_extended_xpub_str(),
      e_child.to_extended_xpub_str()
    );

    let public_descriptor =
      policy::compile(&escrow_public_policy, policy::ScriptType::WSH).unwrap();

    let expected_descriptor = "wsh(multi(2,[c2cb6b81/84h/1h/0h]tpubDDVqM1YixTfp9yZvxxAt5m6ybA48yeeYWdd3KndqrtVRBiPD2PKYMELSTq1JA3qPo4LXimT2VvBCure4JTTvgR3grz8nBY655bgTSCncmSg/*,[9eee95d3/84h/1h/0h]tpubDDmbx25X8ThKHXgMnAj4Tbka85SiisU2KapKp42foekYKYvJ7AiB7MWV5G9wZKpax4fbqaHhuL1MShri2ACA9UDeSmDXyajHQ5ohf8RiUW6/*,[958b4ad7/84h/1h/0h]tpubDD249riYEKPfTZrg9vTdwzmcF3ccLdgHQCp2vx4tuHFV4z6aq42xDZVG1EA3qQwNJkPRaZe6tb3jds65qMRYFgevbd6PXdUPutLbB5JQjft/*))";
    assert_eq!(public_descriptor, expected_descriptor);

    println!("{}", public_descriptor);
    let public_config = WalletConfig::new_offline(&public_descriptor).unwrap();
    let address0 = address::generate(public_config, 0).unwrap().address;
    let expected_address = "tb1q64kehk7zq7xnkhv9m4n800g0tuyn4xrdg68376kgzqsyml2246tq7nu8uq";
    // println!("{:#?}", address0);
    assert_eq!(address0, expected_address);
  }

  /// This test requires funding 5000 sats to the given address
  /// tb1q64kehk7zq7xnkhv9m4n800g0tuyn4xrdg68376kgzqsyml2246tq7nu8uq
  #[test]
  // #[ignore]
  fn test_escrow_wallet_online() {
    let return_address = "mkHS9ne12qx9pS9VojpwU5xtRd4T7X7ZUt";
    let output = psbt::TxOutput{
      address: return_address.to_string(),
      amount: Some(700)
    };
    let expected_address = "tb1q64kehk7zq7xnkhv9m4n800g0tuyn4xrdg68376kgzqsyml2246tq7nu8uq".to_string();
    let _starting_balance = 5_000;

    let public_descriptor = "wsh(multi(2,[c2cb6b81/84h/1h/0h]tpubDDVqM1YixTfp9yZvxxAt5m6ybA48yeeYWdd3KndqrtVRBiPD2PKYMELSTq1JA3qPo4LXimT2VvBCure4JTTvgR3grz8nBY655bgTSCncmSg/*,[9eee95d3/84h/1h/0h]tpubDDmbx25X8ThKHXgMnAj4Tbka85SiisU2KapKp42foekYKYvJ7AiB7MWV5G9wZKpax4fbqaHhuL1MShri2ACA9UDeSmDXyajHQ5ohf8RiUW6/*,[958b4ad7/84h/1h/0h]tpubDD249riYEKPfTZrg9vTdwzmcF3ccLdgHQCp2vx4tuHFV4z6aq42xDZVG1EA3qQwNJkPRaZe6tb3jds65qMRYFgevbd6PXdUPutLbB5JQjft/*))";

    let a_xprv = "[c2cb6b81/84h/1h/0h]tprv8gooCbWUp5z9GWY95JWHgMSs28YCpKTdwL2G3GbYSch2ME8SPzVxAjiaHgCDdyHBLGkUB7Nh5U66G5uLwykSAvECA78Bx6T8mS3wVgQMAGf/*";
    let a_xpub = "[c2cb6b81/84h/1h/0h]tpubDDVqM1YixTfp9yZvxxAt5m6ybA48yeeYWdd3KndqrtVRBiPD2PKYMELSTq1JA3qPo4LXimT2VvBCure4JTTvgR3grz8nBY655bgTSCncmSg/*";

    let b_xprv = "[9eee95d3/84h/1h/0h]tprv8h5Zoc3Gz61eQ4eZtX4U4C6TZ3vnZYH7kHDYXXzNPNx9V4fXUmtavrtcu7nXV253wY741whqjYTUaNR7on91nuB4ydVyfDVbrodzuxRRRQg/*";
    let b_xpub = "[9eee95d3/84h/1h/0h]tpubDDmbx25X8ThKHXgMnAj4Tbka85SiisU2KapKp42foekYKYvJ7AiB7MWV5G9wZKpax4fbqaHhuL1MShri2ACA9UDeSmDXyajHQ5ohf8RiUW6/*";

    let e_xprv = "[958b4ad7/84h/1h/0h]tprv8gL21SgJ5whza6ptGGo3Yb7Vg26gBJVNpuDFeS2bV1T6EVqpCfDN34sPq6rvHgdbfXMP4mUf7khyCRhLpCGCrHqNxDL3g8456KckGU3Q9gW/*";
    let e_xpub = "[958b4ad7/84h/1h/0h]tpubDD249riYEKPfTZrg9vTdwzmcF3ccLdgHQCp2vx4tuHFV4z6aq42xDZVG1EA3qQwNJkPRaZe6tb3jds65qMRYFgevbd6PXdUPutLbB5JQjft/*";

    let a_prv_policy = format!(
      "thresh(2,pk({}),pk({}),pk({}))",
      a_xprv,
      b_xpub,
      e_xpub
    );
    let a_prv_desc = policy::compile(&a_prv_policy, policy::ScriptType::WSH).unwrap();

    let b_prv_policy = format!(
      "thresh(2,pk({}),pk({}),pk({}))",
      a_xpub,
      b_xprv,
      e_xpub
    );
    let b_prv_desc = policy::compile(&b_prv_policy, policy::ScriptType::WSH).unwrap();

    let e_prv_policy = format!(
      "thresh(2,pk({}),pk({}),pk({}))",
      a_xpub,
      b_xpub,
      e_xprv
    );
    let e_prv_desc = policy::compile(&e_prv_policy, policy::ScriptType::WSH).unwrap();

    assert_eq!(
      address::generate(WalletConfig::new_offline(&a_prv_desc).unwrap(),0).unwrap().address,
      address::generate(WalletConfig::new_offline(&b_prv_desc).unwrap(),0).unwrap().address,
    );
    assert_eq!(
      address::generate(WalletConfig::new_offline(&a_prv_desc).unwrap(),0).unwrap().address,
      address::generate(WalletConfig::new_offline(&e_prv_desc).unwrap(),0).unwrap().address,
    );
    assert_eq!(
      address::generate(WalletConfig::new_offline(&a_prv_desc).unwrap(),0).unwrap().address,
      expected_address,
    );

    let _balance = history::sync_balance(
      WalletConfig::new(&public_descriptor,DEFAULT_TESTNET_NODE,None).unwrap()
    ).unwrap();
    
    let policy_id = policy::id(
      WalletConfig::new_offline(&public_descriptor).unwrap()
    ).unwrap();
    assert!(!policy_id.0); // policy path not required
    

    let init_psbt = psbt::build(
      WalletConfig::new(&public_descriptor,DEFAULT_TESTNET_NODE,None).unwrap(), 
      vec![output], 
      300,
      None,
      false
    ).unwrap();
    assert!(!init_psbt.is_finalized);
    
    let a_signed = psbt::sign(
      WalletConfig::new_offline(&a_prv_desc).unwrap(),
      &init_psbt.psbt,
    ).unwrap();
    assert!(!a_signed.is_finalized);

    let b_signed = psbt::sign(
      WalletConfig::new_offline(&b_prv_desc).unwrap(),
      &a_signed.psbt,
    ).unwrap();
    assert!(b_signed.is_finalized);

    // let broadcast = psbt::broadcast(
    //   WalletConfig::new(&public_descriptor,DEFAULT_TESTNET_NODE,None).unwrap(), 
    //   &b_signed.psbt,
    // ).unwrap();
    // assert!(broadcast.txid.len() == 64);

    // let new_balance = history::sync_balance(
    //   WalletConfig::new(&public_descriptor,DEFAULT_TESTNET_NODE,None).unwrap()
    // ).unwrap();

    // println!("Balance: {}", new_balance.balance);

  }

  #[test]
  fn  test_raft_online(){
      let return_address = "mkHS9ne12qx9pS9VojpwU5xtRd4T7X7ZUt";
      let output = psbt::TxOutput{
        address: return_address.to_string(),
        amount: Some(700)
      };
      let expected_address = "tb1q4p6g4cs3e9wwyeg0jfwsrqx6j93h28zzumasvutfv684gmqlkl2qnx0ypf".to_string();
      let timelock = 600_000;
  
      let public_descriptor = "wsh(or_d(pk([c2cb6b81/84h/1h/0h]tpubDDVqM1YixTfp9yZvxxAt5m6ybA48yeeYWdd3KndqrtVRBiPD2PKYMELSTq1JA3qPo4LXimT2VvBCure4JTTvgR3grz8nBY655bgTSCncmSg/*),and_v(v:pk([958b4ad7/84h/1h/0h]tpubDD249riYEKPfTZrg9vTdwzmcF3ccLdgHQCp2vx4tuHFV4z6aq42xDZVG1EA3qQwNJkPRaZe6tb3jds65qMRYFgevbd6PXdUPutLbB5JQjft/*),after(600000))))";
  
      let a_xprv = "[c2cb6b81/84h/1h/0h]tprv8gooCbWUp5z9GWY95JWHgMSs28YCpKTdwL2G3GbYSch2ME8SPzVxAjiaHgCDdyHBLGkUB7Nh5U66G5uLwykSAvECA78Bx6T8mS3wVgQMAGf/*";
      let a_xpub = "[c2cb6b81/84h/1h/0h]tpubDDVqM1YixTfp9yZvxxAt5m6ybA48yeeYWdd3KndqrtVRBiPD2PKYMELSTq1JA3qPo4LXimT2VvBCure4JTTvgR3grz8nBY655bgTSCncmSg/*";

      let e_xprv = "[958b4ad7/84h/1h/0h]tprv8gL21SgJ5whza6ptGGo3Yb7Vg26gBJVNpuDFeS2bV1T6EVqpCfDN34sPq6rvHgdbfXMP4mUf7khyCRhLpCGCrHqNxDL3g8456KckGU3Q9gW/*";
      let e_xpub = "[958b4ad7/84h/1h/0h]tpubDD249riYEKPfTZrg9vTdwzmcF3ccLdgHQCp2vx4tuHFV4z6aq42xDZVG1EA3qQwNJkPRaZe6tb3jds65qMRYFgevbd6PXdUPutLbB5JQjft/*";

      let a_prv_policy = format!(
        "or(pk({}),and(pk({}),after({})))",
        a_xprv,
        e_xpub,
        timelock
      );
      let a_prv_desc = policy::compile(&a_prv_policy, policy::ScriptType::WSH).unwrap();
  
      let e_prv_policy = format!(
        "or(pk({}),and(pk({}),after({})))",
        a_xpub,
        e_xprv,
        timelock
      );
      let e_prv_desc = policy::compile(&e_prv_policy, policy::ScriptType::WSH).unwrap();
  
      assert_eq!(
        address::generate(WalletConfig::new_offline(&a_prv_desc).unwrap(),0).unwrap().address,
        address::generate(WalletConfig::new_offline(&e_prv_desc).unwrap(),0).unwrap().address,
      );
      assert_eq!(
        address::generate(WalletConfig::new_offline(&a_prv_desc).unwrap(),0).unwrap().address,
        address::generate(WalletConfig::new_offline(&public_descriptor).unwrap(),0).unwrap().address,
      );
      assert_eq!(
        address::generate(WalletConfig::new_offline(&a_prv_desc).unwrap(),0).unwrap().address,
        expected_address,
      );
  
      let policy_id = policy::id(
        WalletConfig::new_offline(&public_descriptor).unwrap()
      ).unwrap();
      assert!(policy_id.0); // policy path IS required
      //single sig is only path 0
      //custodian uses path 1

      let init_psbt = psbt::build(
        WalletConfig::new(&public_descriptor,DEFAULT_TESTNET_NODE,None).unwrap(), 
        vec![output.clone()], 
        300,
        None,
        false
      ).unwrap_err();
      assert_eq!(init_psbt.message, "Spending Policy Required".to_string());

      let mut policy_path_both = BTreeMap::new();
      policy_path_both.insert(policy_id.1.clone(), vec![0,1]);

      let init_psbt = psbt::build(
        WalletConfig::new(&public_descriptor,DEFAULT_TESTNET_NODE,None).unwrap(), 
        vec![output], 
        300,
        Some(policy_path_both),
        false,
      ).unwrap();
      assert!(!init_psbt.is_finalized);

      let e_signed = psbt::sign(
        WalletConfig::new_offline(&e_prv_desc).unwrap(),
        &init_psbt.psbt,
      ).unwrap();
      assert!(e_signed.is_finalized);

      let a_signed = psbt::sign(
        WalletConfig::new_offline(&a_prv_desc).unwrap(),
        &init_psbt.psbt,
      ).unwrap();
      assert!(a_signed.is_finalized);
      
  
  }
}
