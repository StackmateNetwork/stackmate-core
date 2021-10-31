
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
  // use crate::config::{DEFAULT_TESTNET_NODE};

  #[test] #[ignore]
  fn test_start_tor() {
    let tor_thread = _start_tor();
    println!("{:#?}", tor_thread);
    let _log = tor_thread.join().unwrap();
  }
}