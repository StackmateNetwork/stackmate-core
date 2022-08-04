///Implementation of BIP392: Methodology for script wallet backups
use std::{str};
use chacha20poly1305::{XChaCha20Poly1305, Key, XNonce};
use chacha20poly1305::aead::{Aead, NewAead};
use secp256k1::rand::{thread_rng,Rng};

pub fn _cc20p1305_encrypt(plaintext:&[u8], key: &[u8])->Result<String,String>{
    let encryption_key = Key::from_slice(key); // 32-bytes
    let aead = XChaCha20Poly1305::new(encryption_key);
    let mut rng = thread_rng();
    let random = rng.gen::<u64>().clone();
    let mut random_string = random.to_string();
    random_string.pop();
    random_string.pop();
    let random_bytes = random_string.as_bytes();
    let nonce = &base64::encode(random_bytes); 
    let nonce = XNonce::from_slice(nonce.as_bytes()); 
    let ciphertext = aead.encrypt(nonce, plaintext).expect("encryption failure!");
    Ok(format!("{}:{}",base64::encode(nonce),base64::encode(&ciphertext).to_string()))
}
pub fn _cc20p1305_decrypt(ciphertext:&str, key: &[u8])->Result<String,String>{
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


#[cfg(test)]
mod tests {
  use super::*;

  // test with external data sources
  #[test]
  fn test_encryption() {
    let message = "thresh(2,wpkh([fingerprint/h/d/path]xpub/*),*,*))";
    let mut rng = thread_rng();
    let random = rng.gen::<u64>().clone();
    let random_string = random.to_string();
    let random_bytes = random_string.as_bytes();
    let key_str = "ishi".to_string() + &base64::encode(random_bytes); 
    println!("KEY: {}", key_str);
    let key = Key::from_slice(&key_str.as_bytes());
    let ciphertext = _cc20p1305_encrypt(message.as_bytes(), &key).unwrap();
    let plaintext = _cc20p1305_decrypt(&ciphertext, key).unwrap();
    println!("{}\n{}",ciphertext,plaintext);
    assert_eq!(&plaintext, message);
  }
}

