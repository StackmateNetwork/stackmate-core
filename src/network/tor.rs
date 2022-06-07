use std::io::Write;
use std::io::{BufRead, BufReader};
use std::net::TcpStream;
use std::path::Path;
use std::str;

// use std::thread::JoinHandle;
use crate::e::{ErrorKind, S5Error};

use libtor::{log, Tor, TorFlag};

use sha1::{Digest, Sha1};

use bitcoin::secp256k1::rand::prelude::thread_rng;
use bitcoin::secp256k1::rand::Rng;

fn _tor_salted_password_hash(password: &str) -> String {
    {
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
    }
}

pub fn start(data_dir: &str, socks5_port: u16, _http_proxy: u16) -> String {
  let is_not_root = data_dir.clone() != "/";  
  let data_dir = Path::new(data_dir);
    let exists = data_dir.exists()  && is_not_root;
    let data_dir = if exists {
        data_dir.join("libstackmate-tor")
    } else {
        eprintln!("Bad base path! using /tmp");
        Path::new("/tmp/libstackmate-tor").to_path_buf()
    };

    // let control_key = aes::keygen();
    // let _hash = tor_salted_password_hash(&control_key);

    match Tor::new()
        .flag(TorFlag::DataDirectory(
            data_dir.to_string_lossy().to_string(),
        ))
        .flag(TorFlag::SocksPort(socks5_port))
        .flag(TorFlag::ControlPort(socks5_port + 1))
        .flag(TorFlag::LogTo(
            log::LogLevel::Err,
            log::LogDestination::Stderr,
        ))
        // .flag(TorFlag::HashedControlPassword(hash))
        // .flag(TorFlag::HiddenServiceDir("/tmp/tor-rust/hs-dir".into()))
        // .flag(TorFlag::HiddenServiceVersion(HiddenServiceVersion::V3))
        // .flag(TorFlag::HiddenServicePort(
        //   TorAddress::Port(8000),
        //   None.into(),
        // ))
        .start(){
            Ok(number)=>number.to_string(),
            Err(e)=>e.to_string(),
        }
}

pub fn bootstrap_progress(control_port: u16, _control_key: &str) -> Result<usize, S5Error> {
    let mut stream = match TcpStream::connect("127.0.0.1:".to_string() + &control_port.to_string())
    {
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
    let progress = if !parts[1].is_empty() {
        parts[1].split(' ').collect::<Vec<&str>>()[2]
    } else {
        "PROGRESS=101"
    };
    // let tag = parts[1].split(" ").collect::<Vec<&str>>()[3];
    // let summary = parts[1].split(" ").collect::<Vec<&str>>()[4];
    let progress_value = progress.split('=').collect::<Vec<&str>>()[1]
        .parse::<usize>()
        .unwrap_or(101);
    println!("PV:{:?}", progress_value);
    Ok(progress_value)
}

pub fn _circuit_established(control_port: u16, _control_key: &str) -> Result<bool, S5Error> {
    let mut stream = match TcpStream::connect("127.0.0.1:".to_string() + &control_port.to_string())
    {
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

pub fn shutdown(control_port: u16, _control_key: &str) -> Result<bool, S5Error> {
    let mut stream = match TcpStream::connect("127.0.0.1:".to_string() + &control_port.to_string())
    {
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
    use std::thread::{spawn, sleep};
    use std::time;
    /// This test might require more than 10 seconds of sleep duration if running for the first time.
    /// Default uses 4 sleep cycles in total for CI. Comment out the last 2 if you have run this before locally.
    #[test] #[ignore]
    fn test_tor() {
        let socks5_port = 31500;
        let http_proxy = 80;
        let control_key = "control_key";
        
        spawn(move|| start("", socks5_port, http_proxy));
        // handle.join().unwrap();
        let duration = time::Duration::from_secs(2);
        sleep(duration);
        sleep(duration);
        println!(
            "{:#?}",
            bootstrap_progress(socks5_port - 100, &control_key).unwrap()
        );
        sleep(duration);
        println!(
            "{:#?}",
            bootstrap_progress(socks5_port - 100, &control_key).unwrap()
        );
        sleep(duration);
        println!(
            "{:#?}",
            bootstrap_progress(socks5_port - 100, &control_key).unwrap()
        );
        sleep(duration);
        println!(
            "{:#?}",
            bootstrap_progress(socks5_port - 100, &control_key).unwrap()
        );
        sleep(duration);
        sleep(duration);
        println!(
            "{:#?}",
            bootstrap_progress(socks5_port - 100, &control_key).unwrap()
        );
        sleep(duration);
        sleep(duration);
        sleep(duration);
        sleep(duration);
        sleep(duration);
        println!(
            "{:#?}",
            bootstrap_progress(socks5_port - 100, &control_key).unwrap()
        );
        assert!(_circuit_established(socks5_port - 100, &control_key).unwrap());
        assert!(shutdown(socks5_port - 100, &control_key).unwrap());
    }
}
