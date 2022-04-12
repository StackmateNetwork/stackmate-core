///Implementation of BIP392: Methodology for script wallet backups
use std::str;
use chacha20poly1305::{XChaCha20Poly1305, Key, XNonce};
use chacha20poly1305::aead::{Aead, NewAead};

pub fn chacha_encrypt(plaintext:&[u8], key: &[u8])->Result<String,String>{
    let encryption_key = Key::from_slice(key); // 32-bytes
    let aead = XChaCha20Poly1305::new(encryption_key);
    let nonce = XNonce::from_slice(b"extra long unique nonce!"); // 24-bytes; unique
    let ciphertext = aead.encrypt(nonce, plaintext).expect("encryption failure!");
    Ok(format!("{}:{}",base64::encode(nonce),base64::encode(&ciphertext).to_string()))
}
pub fn chacha_decrypt(ciphertext:&str, key: &[u8])->Result<String,String>{
    let encryption_key = Key::from_slice(key); // 32-bytes
    let aead = XChaCha20Poly1305::new(encryption_key);
    let iter:Vec<&str> = ciphertext.split(":").collect();
    let nonce_slice = &base64::decode(&iter[0].as_bytes()).unwrap();
    let nonce = XNonce::from_slice(nonce_slice); // 24-bytes; unique
    let ciphertext_bytes: &[u8] = &base64::decode(&iter[1].as_bytes()).unwrap();
    let plaintext = aead.decrypt(nonce, ciphertext_bytes).expect("decryption failure!");
    match str::from_utf8(&plaintext){
        Ok(message)=>Ok(message.to_string()),
        Err(_)=>Err("Bad Text".to_string())
    }
}
// pub fn share(destination: &str, access_token: &str)->Result<String,String>;

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_encryption() {
    let message = "thresh(2,wpkh([fingerprint/h/d/path]xpub/*),*,*))";
    let key = Key::from_slice(b"an example very very secret key."); // 32-bytes
    let ciphertext = chacha_encrypt(message.as_bytes(), &key).unwrap();
    let plaintext = chacha_decrypt(&ciphertext, key).unwrap();
    println!("{}",plaintext);
    assert_eq!(&plaintext, message);
  }
}

