use std::io::prelude::*;
use std::io::{self, Read, Write};
use std::io::{BufReader, BufWriter};
use std::net::TcpStream;
use std::str;
use std::thread::sleep;
use std::thread::JoinHandle;
use std::time::Duration;

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

fn _status() -> String {
  let mut stream = TcpStream::connect("127.0.0.1:9000").unwrap();
  // stream
  //   .set_read_timeout(Some(Duration::from_millis(3000)))
  //   .unwrap();

  stream.write(b"AUTHENTICATE").unwrap();
  stream.write(b"\r\n").unwrap();

  // stream.write(b"GETINFO process/pid").unwrap();
  // stream.write(b"\r\n").unwrap();

  let mut reader = io::BufReader::new(&mut stream);

  let received: Vec<u8> = reader.fill_buf().unwrap().to_vec();

  // Mark the bytes read as consumed so the buffer will not return them in a subsequent read
  reader.consume(received.len());

  str::from_utf8(&received).unwrap().to_string()
  
}

fn _off() -> String {
  let mut stream = TcpStream::connect("127.0.0.1:9000").unwrap();

  stream.write(b"AUTHENTICATE").unwrap();
  stream.write(b"\r\n").unwrap();

  stream.write(b"SIGNAL SHUTDOWN").unwrap();
  stream.write(b"\r\n").unwrap();
  let mut reader = io::BufReader::new(&mut stream);

  let received: Vec<u8> = reader.fill_buf().unwrap().to_vec();

  // Mark the bytes read as consumed so the buffer will not return them in a subsequent read
  reader.consume(received.len());

  str::from_utf8(&received).unwrap().to_string()
  
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::config::{WalletConfig, DEFAULT_TESTNET_NODE};
  use crate::network::fees;
  use std::thread::sleep;
  use std::time;
  /// This test might require more than 10 seconds of sleep duration if running for the first time.
  /// Use longer duration if required.
  #[test]
  fn test_tor() {
    _on();
    // println!("{:#?}", tor_thread.thread().id());
    // let _log = tor_thread.join().unwrap();
    let duration = time::Duration::from_secs(5);
    sleep(duration);
    let deposit_desc = "/0/*";
    let config = WalletConfig::new(
      deposit_desc,
      DEFAULT_TESTNET_NODE,
      Some("127.0.0.1:19050".to_string()),
    )
    .unwrap();
    let fees = fees::estimate_rate(config, 6).unwrap();
    println!("{:#?}", fees);

    println!("{:#?}", _status());
    println!("{:#?}", _off());

    sleep(time::Duration::from_secs(5))
  }
}
