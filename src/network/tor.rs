use std::io::{ Write};
use std::io::{BufReader, BufRead};
use std::net::TcpStream;
use std::str;
use std::thread::JoinHandle;
use crate::e::{ErrorKind, S5Error};
use libtor::{Error, HiddenServiceVersion, Tor, TorAddress, TorFlag, log};

fn start() -> JoinHandle<Result<u8, Error>> {
  Tor::new()
    .flag(TorFlag::DataDirectory("/tmp/tor-rust".into()))
    .flag(TorFlag::SocksPort(19050))
    .flag(TorFlag::ControlPort(9000))
    .flag(TorFlag::LogTo(log::LogLevel::Err,log::LogDestination::Stderr))
    // .flag(TorFlag::HiddenServiceDir("/tmp/tor-rust/hs-dir".into()))
    // .flag(TorFlag::HiddenServiceVersion(HiddenServiceVersion::V3))
    // .flag(TorFlag::HiddenServicePort(
    //   TorAddress::Port(8000),
    //   None.into(),
    // ))
    .start_background()
  
}

fn bootstrap_progress() -> Result<String, S5Error> {
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
  let _result = stream.write(b"AUTHENTICATE").unwrap();
  let _result = stream.write(b"\r\n").unwrap();
  let _result = stream.write(b"GETINFO status/bootstrap-phase").unwrap();
  let _result = stream.write(b"\r\n").unwrap();

  let mut reader = BufReader::new(&mut stream);
  let received: Vec<u8> = reader.fill_buf().unwrap().to_vec();
  // Mark the bytes read as consumed so the buffer will not return them in a subsequent read
  reader.consume(received.len());

  let response_string = str::from_utf8(&received).unwrap().to_string();
  stream.flush().unwrap();
  Ok(response_string)
}

fn circuit_established() -> Result<bool, S5Error> {
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
  let _result = stream.write(b"AUTHENTICATE").unwrap();
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

fn shutdown() -> Result<bool, S5Error> {
  let mut stream = match TcpStream::connect("127.0.0.1:9000") {
    Ok(result) => result,
    Err(_) => {
      return Err(S5Error::new(
        ErrorKind::Network,
        "Could not connect to tor daemon.",
      ))
    }
  };

  let _result = stream.write(b"AUTHENTICATE").unwrap();
  let _result = stream.write(b"\r\n").unwrap();
  let _result = stream.flush().unwrap();
  let _result = stream.write(b"SIGNAL SHUTDOWN").unwrap();
  let _result = stream.write(b"\r\n").unwrap();

  let mut reader = BufReader::new(&mut stream);
  let received: Vec<u8> = reader.fill_buf().unwrap().to_vec();
  // Mark the bytes read as consumed so the buffer will not return them in a subsequent read
  reader.consume(received.len());

  let response_str = str::from_utf8(&received).unwrap();
  println!("{}", response_str);
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
    let _handle = start();
    // handle.join().unwrap();

    let duration = time::Duration::from_secs(1);
    
    sleep(duration);
    sleep(duration);

    println!("{:#?}", bootstrap_progress().unwrap());
    
    sleep(duration);
    println!("{:#?}", bootstrap_progress().unwrap());

    sleep(duration);
    println!("{:#?}", bootstrap_progress().unwrap());

    sleep(duration);
    println!("{:#?}", bootstrap_progress().unwrap());

    let deposit_desc = "/0/*";
    let config = WalletConfig::new(
      deposit_desc,
      DEFAULT_MAINNET_NODE,
      Some("127.0.0.1:19050".to_string()),
    )
    .unwrap();
    let fees = fees::estimate_rate(config, 6).unwrap();
    assert!(fees.rate > 0.1);

    assert!( circuit_established().unwrap());
    assert!(shutdown().unwrap());

    // sleep(time::Duration::from_secs(5))
  }
}
