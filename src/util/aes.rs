use bitcoin::base64;
use bitcoin::secp256k1::rand::prelude::thread_rng;
use bitcoin::secp256k1::rand::Rng;
use openssl::symm;
use std::str;

/// Keys are 32 bytes long hex encoded.
pub fn keygen() -> String {
  let key: Vec<u8> = thread_rng().gen::<[u8; 32]>().to_vec();
  hex::encode(key)
}

/// String wrapper for AES-256-CBC encrypt w/iv
/// Cipher text is base64 encoded
/// input Plaintext, outputs IVEString
pub fn encrypt(plaintext: &str, key: &str) -> String {
  let iv = thread_rng().gen::<[u8; 16]>().to_vec();
  let cipher = symm::Cipher::aes_256_cbc();
  let ciphertext = symm::encrypt(
    cipher,
    &hex::decode(key).unwrap(),
    Some(&iv),
    plaintext.as_bytes(),
  )
  .unwrap();
  base64::encode(&iv) + &String::from(':') + &base64::encode(&ciphertext).to_string()
}

/// String wrapper for AES-256-CBC decrypt w/iv
/// Cipher text is base64 encoded
/// input IVEString, outputs Plaintext
pub fn decrypt(iv_ciphertext: &str, key: &str) -> String {
  let cipher = symm::Cipher::aes_256_cbc();
  let iter: Vec<&str> = iv_ciphertext.split(':').collect();
  // println!("{}, {}", iter[0], iter[1]);
  // println!("KEY: {}", key);

  let plaintext = symm::decrypt(
    cipher,
    &hex::decode(key).unwrap(),
    Some(&base64::decode(iter[0]).unwrap()),
    &base64::decode(iter[1]).unwrap(),
  )
  .unwrap();

  str::from_utf8(&plaintext).unwrap().to_string()
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_keygen() {
    let key = keygen();
    println!(
      "FRESH KEY:  {}  :: length={}",
      key,
      key.len()
    );
    // println!("TOR HASHED: {}", hashed)
  }

  #[test]
  fn test_aes() {
    let secret = "thesecretsauce";
    let key = keygen();
    let iv_ciphertext = encrypt(secret.clone(), &key.clone());
    // println!("IV ENCRYPTED SECRET:  {}",&iv_ciphertext);
    let plaintext = decrypt(&iv_ciphertext.clone(), &key.clone());
    // println!("IV DECRYPTED SECRET:  {}",&plaintext);
    assert_eq!(secret, plaintext)
  }
}
