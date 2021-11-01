
use std::thread::{JoinHandle};
use libtor::{Tor, TorFlag, TorAddress, HiddenServiceVersion, Error};

fn _start_tor()->JoinHandle<Result<u8,Error>>{
  Tor::new()
  .flag(TorFlag::DataDirectory("/tmp/tor-rust".into()))
  .flag(TorFlag::SocksPort(19050))
  .flag(TorFlag::HiddenServiceDir("/tmp/tor-rust/hs-dir".into()))
  .flag(TorFlag::HiddenServiceVersion(HiddenServiceVersion::V3))
  .flag(TorFlag::HiddenServicePort(TorAddress::Port(8000), None.into()))
  .start_background()
}

#[cfg(test)]
mod tests {
  use super::*;
  use std::thread::sleep;
  use std::time;
  use crate::config::{WalletConfig, DEFAULT_TESTNET_NODE};
  use crate::network::fees;
  /// This test might require more than 10 seconds of sleep duration if running for the first time.
  /// Use longer duration if required.
  #[test] #[ignore]
  fn test_start_tor() {
    let tor_thread = _start_tor();
    println!("{:#?}", tor_thread.thread().id());
    // let _log = tor_thread.join().unwrap();
    let duration = time::Duration::from_secs(10);
    sleep(duration);
    let deposit_desc = "/0/*";
    let config = WalletConfig::new(
      deposit_desc, 
      DEFAULT_TESTNET_NODE, 
      Some("127.0.0.1:19050".to_string())
    ).unwrap();
    let fees = fees::estimate_rate(config,6).unwrap();
    println!("{:#?}", fees);

  }
}