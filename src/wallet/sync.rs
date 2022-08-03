use crate::config::WalletConfig;
use crate::e::{ErrorKind, S5Error};
use bdk::database::SqliteDatabase;
use bdk::{Wallet, SyncOptions};
pub fn sqlite(config: WalletConfig) -> Result<(), S5Error> {
    if config.db_path.is_none(){
        return Err(S5Error::new(ErrorKind::Input, "SQLite Requires a Db Path."));
    }
    if config.client.is_none(){
        return Err(S5Error::new(ErrorKind::Input, "SQLite Sync Requires a Blockchain Client"));
    }

    let wallet = match Wallet::new(
        &config.deposit_desc,
        Some(&config.change_desc),
        config.network,
        SqliteDatabase::new(config.db_path.unwrap()),
    ) {
        Ok(result) => result,
        Err(e) => return Err(S5Error::new(ErrorKind::Internal, &e.to_string())),
    };

    match wallet.sync(&config.client.unwrap(),SyncOptions::default()) {
        Ok(_) => 
            Ok(())
        ,
        Err(e) => Err(S5Error::new(ErrorKind::Internal, &e.to_string())),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{env, path::Path};
    use crate::config::{DEFAULT_TESTNET_NODE};
    use std::fs;
    use secp256k1::rand::{thread_rng,Rng};

    #[test]
    fn test_sqlite() {
        let xkey = "[db7d25b5/84'/1'/6']tpubDCCh4SuT3pSAQ1qAN86qKEzsLoBeiugoGGQeibmieRUKv8z6fCTTmEXsb9yeueBkUWjGVzJr91bCzeCNShorbBqjZV4WRGjz3CrJsCboXUe";
        let descriptor = format!("wpkh({}/*)", xkey);
        let mut rng = thread_rng();
        let random: u16 = rng.gen();
        let db_path: String = env::var("CARGO_MANIFEST_DIR").unwrap() + &random.to_string() + ".db";
        let config = WalletConfig::new(&descriptor, DEFAULT_TESTNET_NODE, None,Some(db_path.clone())).unwrap();        
        let status = sqlite(config);
        assert_eq!(
            (),
            status.unwrap()
        );
        fs::remove_file(Path::new(&db_path))
        .expect("File delete failed");
    }
}
