use std::io::{ Write};
use std::io::{BufReader, BufRead};
use std::net::TcpStream;
use std::str;
use std::path::Path;

// use std::thread::JoinHandle;
use crate::e::{ErrorKind, S5Error};
use crate::util::aes;

use libtor::{Tor,TorFlag,log};

use sha1::{Sha1, Digest};

use bitcoin::secp256k1::rand::prelude::thread_rng;
use bitcoin::secp256k1::rand::Rng;


fn tor_salted_password_hash(password: &str)->String{{
  let salt = thread_rng().gen::<[u8; 8]>().to_vec();
  let salt_hex = hex::encode(salt);
  let to_hash = salt_hex.clone() + password;

  let mut hasher = Sha1::new();
  hasher.update(to_hash.as_bytes());
  let hashed_password = hasher.finalize();
  let mut hashed_password_hex = hex::encode(&hashed_password);
  hashed_password_hex.insert_str(0, &salt_hex);
  hashed_password_hex.insert_str(0, "16:");
  hashed_password_hex
}}


pub fn start(tmp_path: &str) -> String {
  let tmp_path = Path::new(tmp_path);
  let exists = tmp_path.exists();
  let tmp_path = if exists {
    tmp_path.join("tor-rust")
  } else {
    eprintln!("Bad base path! using /tmp");
    Path::new("/tmp/tor-rust").to_path_buf()
  };
    
  let control_key = aes::keygen();
  let _hash = tor_salted_password_hash(&control_key);

  Tor::new()
    .flag(TorFlag::DataDirectory(tmp_path.to_string_lossy().to_string()))
    .flag(TorFlag::SocksPort(19050))
    .flag(TorFlag::ControlPort(9000))
    .flag(TorFlag::LogTo(log::LogLevel::Err,log::LogDestination::Stderr))
    // .flag(TorFlag::HashedControlPassword(hash))
    // .flag(TorFlag::HiddenServiceDir("/tmp/tor-rust/hs-dir".into()))
    // .flag(TorFlag::HiddenServiceVersion(HiddenServiceVersion::V3))
    // .flag(TorFlag::HiddenServicePort(
    //   TorAddress::Port(8000),
    //   None.into(),
    // ))
    .start_background();
  "control_key".to_string()
}

pub fn bootstrap_progress(_control_key: &str) -> Result<usize, S5Error> {
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
  let _result = stream.write(b"AUTHENTICATE ").unwrap();
  // let _result = stream.write(control_key.as_bytes()).unwrap();
  let _result = stream.write(b"\r\n").unwrap();
  let _result = stream.write(b"GETINFO status/bootstrap-phase").unwrap();
  let _result = stream.write(b"\r\n").unwrap();

  let mut reader = BufReader::new(&mut stream);
  let received: Vec<u8> = reader.fill_buf().unwrap().to_vec();
  // Mark the bytes read as consumed so the buffer will not return them in a subsequent read
  reader.consume(received.len());

  let response_string = str::from_utf8(&received).unwrap().to_string();
  stream.flush().unwrap();
  let parts = response_string.split("\r\n").collect::<Vec<&str>>();
  println!("{:?}", parts[1]);
  let progress = if parts[1] != "" {parts[1].split(" ").collect::<Vec<&str>>()[2]}else{"PROGRESS=101"};
  // let tag = parts[1].split(" ").collect::<Vec<&str>>()[3];
  // let summary = parts[1].split(" ").collect::<Vec<&str>>()[4];
  let progress_value = progress.split("=").collect::<Vec<&str>>()[1].parse::<usize>().unwrap_or(101);
  println!("PV:{:?}", progress_value);
  Ok(progress_value)
}

pub fn circuit_established(_control_key: &str) -> Result<bool, S5Error> {
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
  let _result = stream.write(b"AUTHENTICATE ").unwrap();
  // let _result = stream.write(control_key.as_bytes()).unwrap();
  let _result = stream.write(b"\r\n").unwrap();
  let _result = stream.write(b"GETINFO status/circuit-established").unwrap();
  let _result = stream.write(b"\r\n").unwrap();

  let mut reader = BufReader::new(&mut stream);
  let received: Vec<u8> = reader.fill_buf().unwrap().to_vec();
  // Mark the bytes read as consumed so the buffer will not return them in a subsequent read
  reader.consume(received.len());

  let response_str = str::from_utf8(&received).unwrap();
  Ok(response_str.contains("circuit-established=1"))
}

pub fn shutdown(_control_key: &str) -> Result<bool, S5Error> {
  let mut stream = match TcpStream::connect("127.0.0.1:9000") {
    Ok(result) => result,
    Err(_) => {
      return Err(S5Error::new(
        ErrorKind::Network,
        "Could not connect to tor daemon.",
      ))
    }
  };

  let _result = stream.write(b"AUTHENTICATE ").unwrap();
  // let _result = stream.write(control_key.as_bytes()).unwrap();
  let _result = stream.write(b"\r\n").unwrap();
  let _result = stream.flush().unwrap();
  let _result = stream.write(b"SIGNAL SHUTDOWN").unwrap();
  let _result = stream.write(b"\r\n").unwrap();

  let mut reader = BufReader::new(&mut stream);
  let received: Vec<u8> = reader.fill_buf().unwrap().to_vec();
  // Mark the bytes read as consumed so the buffer will not return them in a subsequent read
  reader.consume(received.len());

  let response_str = str::from_utf8(&received).unwrap();
  Ok(response_str.contains("250 OK\r\n"))
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
  #[test]
  fn test_tor() {
    let control_key = start("/tmp");
    // handle.join().unwrap();

    let duration = time::Duration::from_secs(1);
    
    sleep(duration);
    sleep(duration);

    println!("{:#?}", bootstrap_progress(&control_key).unwrap());
    
    sleep(duration);
    println!("{:#?}", bootstrap_progress(&control_key).unwrap());

    sleep(duration);
    println!("{:#?}", bootstrap_progress(&control_key).unwrap());

    sleep(duration);
    println!("{:#?}", bootstrap_progress(&control_key).unwrap());

    sleep(duration);
    println!("{:#?}", bootstrap_progress(&control_key).unwrap());

    sleep(duration);
    println!("{:#?}", bootstrap_progress(&control_key).unwrap());

    let deposit_desc = "/0/*";
    let config = WalletConfig::new(
      deposit_desc,
      DEFAULT_MAINNET_NODE,
      Some("127.0.0.1:19050".to_string()),
    )
    .unwrap();
    let fees = fees::estimate_rate(config, 6).unwrap();
    assert!(fees.rate > 0.1);

    assert!( circuit_established(&control_key).unwrap());
    assert!(shutdown(&control_key).unwrap());
  }
}
