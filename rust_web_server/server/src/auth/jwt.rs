use jwt::{AlgorithmType, PKeyWithDigest, SignWithKey, Token};
use openssl::{
    hash::MessageDigest,
    pkey::{PKey, Private},
    rsa::Rsa,
};
use serde::{Deserialize, Serialize};
use std::error::Error;

pub struct Key {
    pkey_and_digest: PKeyWithDigest<Private>,
}

impl Key {
    pub fn new() -> Result<Key, Box<dyn Error>> {
        let rsa = Rsa::generate(4096)?;
        let pkey = PKey::from_rsa(rsa)?;

        let public_key = pkey.public_key_to_pem()?;
        let public_key_str = std::str::from_utf8(public_key.as_slice())?;
        trace!("JWT public key\n{public_key_str}");

        let private_key = pkey.private_key_to_pem_pkcs8()?;
        let private_key_str = std::str::from_utf8(private_key.as_slice())?;
        trace!("JWET private key\n{private_key_str}");

        Ok(Key {
            pkey_and_digest: PKeyWithDigest {
                digest: MessageDigest::sha512(),
                key: pkey,
            },
        })
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub username: String,
}

impl Claims {
    // TODO should expire
    pub fn to_jwt(&self, key: &Key) -> Result<String, Box<dyn Error>> {
        Ok(Token::new(
            jwt::Header {
                algorithm: AlgorithmType::Rs512,
                ..Default::default()
            },
            self,
        )
        .sign_with_key(&key.pkey_and_digest)?
        .as_str()
        .to_string())
    }

    pub fn from_jwt(key: &Key, jwt: &str) -> Self {
        todo!();
    }
}
