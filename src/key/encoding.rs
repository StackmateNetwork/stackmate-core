use std::str::FromStr;

pub enum Encoding {
    Base64,
    Base32,
    Hex
}
impl FromStr for Encoding{
    type Err = ();

    fn from_str(s: &str) -> Result<Encoding,Self::Err>{
        let level = match s {
            "hex"=>Encoding::Hex,
            "base32"=>Encoding::Base32,
            "base64"=>Encoding::Base64,
            &_=>return Err(())
        };
        Ok(level)
    }
}
