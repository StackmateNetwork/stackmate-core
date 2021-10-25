use std::fmt::Display;
use std::fmt::Formatter;
use std::os::raw::c_char;
use std::ffi::{CString};

use serde::{Serialize,Deserialize};

#[derive(Serialize,Deserialize,Debug,Copy,Clone)]
pub enum ErrorKind {
  KeyError,
  WalletError,
  NetworkError,
  InputError,
  OpError,
}

impl Display for ErrorKind {
  fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
    match self {
      ErrorKind::InputError => write!(f, "InputError"),
      ErrorKind::OpError => write!(f, "OpError"),
      ErrorKind::KeyError => write!(f, "KeyError"),
      ErrorKind::WalletError => write!(f, "WalletError"),
      ErrorKind::NetworkError => write!(f, "NetworkError"),

    }
  }
}

#[derive(Serialize,Deserialize,Debug,Clone)]
pub struct S5Error {
  pub kind: String,
  pub message: String,
}

impl S5Error {
  pub fn new(kind: ErrorKind, message: &str) -> Self {
    S5Error {
      kind: kind.to_string(),
      message: message.to_string(),
    }
  }
  pub fn c_stringify(&self)->*mut c_char{
    let stringified = match serde_json::to_string(&self.clone()){
        Ok(result)=>result,
        Err(_)=>return CString::new("Error:JSON Stringify Failed. BAD NEWS! Contact Support.").unwrap().into_raw()
    };

    CString::new(stringified).unwrap().into_raw()
  }
}
