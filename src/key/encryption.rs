// ///Implementation of BIP392: Methodology for script wallet backups
// use chacha20poly1305::{XChaCha20Poly1305, Key, XNonce};
// use chacha20poly1305::aead::{Aead, NewAead};

// use crate::key::derivation;
// use crate::key::ec;
// use crate::key::seed;
// use crate::wallet::policy;

// // pub fn keygen(bip39_mnemonic: &str, bip32_path: &str)->Result<String,String>;
// // pub fn encrypt(msg:&str, key: &str)->Result<String,String>;
// // pub fn decrypt(cipher: &str, key: &str)->Result<String,String>;
// // pub fn share(destination: &str, access_token: &str)->Result<String,String>;

// #[cfg(test)]
// mod tests {
//   use super::*;
//   #[test]
//   fn test_encryption() {
//     let message = b"thresh(2,wpkh([fingerprint/h/d/path]xpub/*),*,*))";
//     let key = Key::from_slice(b"an example very very secret key."); // 32-bytes
//     let aead = XChaCha20Poly1305::new(key);

//     let nonce = XNonce::from_slice(b"extra long unique nonce!"); // 24-bytes; unique
//     let ciphertext = aead.encrypt(nonce, message.as_ref()).expect("encryption failure!");
//     println!("{:#?}:{:#?}",nonce,ciphertext.clone());
//     let plaintext = aead.decrypt(nonce, ciphertext.as_ref()).expect("decryption failure!");
//     assert_eq!(&plaintext, message);
//   }
// }

