// use openssl::symm;
// use std::str;
// use bitcoin::secp256k1::rand::prelude::*;
// // extern crate hex;

// use crate::key::encoding::Encoding;

// // use rand_seeder::{Seeder};
// // use rand_pcg::Pcg64;

// /// Create a random 256 bit key to use for aes encryption
// pub fn keygen(encoding: Encoding)->String{
//     let key: Vec<u8> =thread_rng().gen::<[u8; 32]>().to_vec();
//     match encoding  {
//         Encoding::Base64=>base64::encode(key),
//         Encoding::Base32=>base32::encode(base32::Alphabet::RFC4648 {padding: false}, &key),
//         Encoding::Hex=>hex::encode(key)
//     }

// }

// /// Create a seeded 256 bit key to use for aes encryption
// // pub fn seedgen(seed:String)->String{
// //     let mut key: Pcg64 = Seeder::from(seed.as_str()).make_rng();
// //     base64::encode(key.gen::<[u8; 32]>())
// // }

// /// String wrapper for AES-256-CBC encrypt 
// pub fn cbc_encrypt(plaintext:&str, key: &str, encoding: Encoding)->String{
//     let iv = thread_rng().gen::<[u8; 16]>().to_vec();
//     let cipher = symm::Cipher::aes_256_cbc();
//     let ciphertext = symm::encrypt(
//         cipher,
//         &base64::decode(key).unwrap(),
//         Some(&iv),
//         plaintext.as_bytes()
//     ).unwrap();
//     match encoding{
//       Encoding::Base32=>base32::encode(base32::Alphabet::RFC4648 {padding: false}, &iv).to_string() + ":" + &base32::encode(base32::Alphabet::RFC4648 {padding: false}, &ciphertext),
//       Encoding::Hex=>hex::encode(&iv) + ":" + &hex::encode(ciphertext),
//       Encoding::Base64=>base64::encode(&iv).to_string() + &String::from(":") + &base64::encode(&ciphertext),
//     }

// }

// /// String wrapper for AES-256-CBC decrypt 
// pub fn cbc_decrypt(iv_ciphertext:&str, key: &str)->String{
//     let cipher = symm::Cipher::aes_256_cbc();
//     let iter:Vec<&str> = iv_ciphertext.split(":").collect();
//     let plaintext = symm::decrypt(
//         cipher,
//         &base64::decode(key).unwrap(),
//         Some(&base64::decode(iter[0]).unwrap()),
//         &base64::decode(iter[1]).unwrap()
//     ).unwrap();

//     str::from_utf8(&plaintext).unwrap().to_string()
// }


// #[cfg(test)]
// mod tests {
//     use super::*;

//     #[test]
//     fn test_keygen(){
//         println!("FRESH RANDOM KEY:  {}  :: length={}",keygen(Encoding::Base64), keygen(Encoding::Base64).len());
//         println!("FRESH RANDOM KEY:  {}  :: length={}",keygen(Encoding::Hex), keygen(Encoding::Hex).len())

//     }

//     // #[test]
//     // fn test_seedgen(){
//     //     println!("SEEDED KEY:  {}",seedgen(String::from("myseed")))
//     // }

//     #[test]
//     fn test_encrypt_decrypt(){
//         let secret = "I am very much interested torustyou";
//         let key = "a79FAWI1IKtuwoSoT3hq0lfkq0oxchoHy1xhOTSpHaU=";
//         let iv_ciphertext = cbc_encrypt(secret.clone(),key.clone(),Encoding::Base64);
//         println!("IV ENCRYPTED SECRET:  {}",&iv_ciphertext);
//         let plaintext = cbc_decrypt(&iv_ciphertext.clone(), key.clone());
//         println!("IV DECRYPTED SECRET:  {}",&plaintext);
//         assert_eq!(secret,plaintext)
//     }
// }
