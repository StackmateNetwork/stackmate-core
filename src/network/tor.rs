use std::io::prelude::*;
use std::io::{self, Read, Write};
use std::io::{BufReader, BufWriter};
use std::net::TcpStream;
use std::str;
use std::thread::sleep;
use std::thread::JoinHandle;
use std::time::Duration;

use crate::e::{ErrorKind, S5Error};
use libtor::{Error, HiddenServiceVersion, Tor, TorAddress, TorFlag};

fn _on() -> JoinHandle<Result<u8, Error>> {
  Tor::new()
    .flag(TorFlag::DataDirectory("/tmp/tor-rust".into()))
    .flag(TorFlag::SocksPort(19050))
    .flag(TorFlag::ControlPort(9000))
    .flag(TorFlag::HiddenServiceDir("/tmp/tor-rust/hs-dir".into()))
    .flag(TorFlag::HiddenServiceVersion(HiddenServiceVersion::V3))
    .flag(TorFlag::HiddenServicePort(
      TorAddress::Port(8000),
      None.into(),
    ))
    .start_background()
}

fn _status() -> Result<bool, S5Error> {
  let mut stream = match TcpStream::connect("127.0.0.1:9000") {
    Ok(result) => result,
    Err(_) => {
      return Err(S5Error::new(
        ErrorKind::Network,
        "Could not connect to tor daemon.",
      ))
    }
  };
  // stream
  //   .set_read_timeout(Some(Duration::from_millis(3000)))
  //   .unwrap();

  stream.write(b"AUTHENTICATE").unwrap();
  stream.write(b"\r\n").unwrap();

  stream.write(b"GETINFO status/circuit-established").unwrap();
  stream.write(b"\r\n").unwrap();

  let mut reader = io::BufReader::new(&mut stream);

  let received: Vec<u8> = reader.fill_buf().unwrap().to_vec();

  // Mark the bytes read as consumed so the buffer will not return them in a subsequent read
  reader.consume(received.len());
  let response_str = str::from_utf8(&received).unwrap();
  Ok(response_str.contains("circuit-established=1"))
}

fn _off() -> Result<bool, S5Error> {
  let mut stream = match TcpStream::connect("127.0.0.1:9000") {
    Ok(result) => result,
    Err(_) => {
      return Err(S5Error::new(
        ErrorKind::Network,
        "Could not connect to tor daemon.",
      ))
    }
  };

  stream.write(b"AUTHENTICATE").unwrap();
  stream.write(b"\r\n").unwrap();
  stream.flush().unwrap();
  stream.write(b"SIGNAL SHUTDOWN").unwrap();
  stream.write(b"\r\n").unwrap();
  let mut reader = io::BufReader::new(&mut stream);

  let received: Vec<u8> = reader.fill_buf().unwrap().to_vec();

  // Mark the bytes read as consumed so the buffer will not return them in a subsequent read
  reader.consume(received.len());

  let response_str = str::from_utf8(&received).unwrap();
  println!("{}", response_str);
  Ok(response_str == "250 OK\r\n")
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::config::{WalletConfig, DEFAULT_MAINNET_NODE};
  use crate::network::fees;
  use std::thread::sleep;
  use std::time;
  /// This test might require more than 10 seconds of sleep duration if running for the first time.
  /// Default uses 4 sleep cycles in total for CI. Comment out the last 2 if you have run this before locally.
  #[test] #[ignore]
  fn test_tor() {
    _on();
    // println!("{:#?}", tor_thread.thread().id());
    // let _log = tor_thread.join().unwrap();
    assert!(_status().err().is_some());
    let duration = time::Duration::from_secs(3);
    
    sleep(duration);

    assert!(!_status().unwrap());
    
    sleep(duration);
    sleep(duration);
    sleep(duration);

    let deposit_desc = "/0/*";
    let config = WalletConfig::new(
      deposit_desc,
      DEFAULT_MAINNET_NODE,
      Some("127.0.0.1:19050".to_string()),
    )
    .unwrap();
    let fees = fees::estimate_rate(config, 6).unwrap();
    assert!(fees.rate > 0.1);

    assert!( _status().unwrap());
    assert!(_off().unwrap());

    sleep(time::Duration::from_secs(5))
  }
}
