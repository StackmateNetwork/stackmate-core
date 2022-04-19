///Implementation of BIP392: Methodology for script wallet backups

#[cfg(test)]
mod tests {
  use std::str::FromStr;
  use bitcoin::util::bip32::ExtendedPrivKey;
  use bitcoin::network::constants::Network;
  use secp256k1::hashes::sha256;
  use secp256k1::{Message};
  use chacha20poly1305::{XChaCha20Poly1305, Key, XNonce};
  use chacha20poly1305::aead::{Aead, NewAead};
  use crate::key::derivation;
  use crate::key::seed;
//   use crate::wallet::policy;
  #[test]
  fn test_bip392() {
    let message = b"thresh(2,wpkh([fingerprint/h/d/path]xpub/*),*,*))";
    let seed = seed::generate(
        12, 
        "script ready", 
        Network::Testnet
    ).unwrap();
    let encryption_key_source = derivation::to_hardened_account(
        &seed.xprv, 
        derivation::DerivationPurpose::Encryption, 
        0)
    .unwrap();
    let encryption_xkey = ExtendedPrivKey::from_str(&encryption_key_source.xprv).unwrap();
    let encryption_key_pre_image: &[u8] = &encryption_xkey.private_key.to_bytes();
    let encryption_key = Message::from_hashed_data::<sha256::Hash>(encryption_key_pre_image);
    let key = Key::from_slice(encryption_key.as_ref()); // 32-bytes
    let aead = XChaCha20Poly1305::new(key);
    let nonce = XNonce::from_slice(b"extra long unique nonce!"); // 24-bytes; unique
    let ciphertext = aead.encrypt(
        nonce, 
        message.as_ref()
    ).expect("encryption failure!");
    println!("This is what erd looks like:\n{:#?}:{:#?}\nSTORE IT EVERYWHERE!",base64::encode(nonce),base64::encode(ciphertext.clone()));
    let plaintext = aead.decrypt(
        nonce, 
        ciphertext.as_ref()
    ).expect("decryption failure!");
    assert_eq!(&plaintext,message);
  }
}

