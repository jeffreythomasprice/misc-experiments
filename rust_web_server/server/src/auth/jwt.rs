use jwt::{AlgorithmType, PKeyWithDigest, SignWithKey, Token, VerifyWithKey};
use openssl::{
    hash::MessageDigest,
    pkey::{PKey, Private, Public},
    rsa::Rsa,
};
use serde::{Deserialize, Serialize};
use std::error::Error;

pub struct Key {
    private: PKeyWithDigest<Private>,
    public: PKeyWithDigest<Public>,
}

impl Key {
    pub fn new() -> Result<Key, Box<dyn Error>> {
        // TODO persist keys so we don't get invalid signature verifying every time we reboot the server
        let rsa = Rsa::generate(4096)?;
        let pkey = PKey::from_rsa(rsa)?;

        let public_key = pkey.public_key_to_pem()?;
        let public_key_str = std::str::from_utf8(public_key.as_slice())?;
        trace!("JWT public key\n{public_key_str}");

        let private_key = pkey.private_key_to_pem_pkcs8()?;
        let private_key_str = std::str::from_utf8(private_key.as_slice())?;
        trace!("JWT private key\n{private_key_str}");

        let digest = MessageDigest::sha512();
        Ok(Key {
            private: PKeyWithDigest { digest, key: pkey },
            public: PKeyWithDigest {
                digest,
                key: PKey::public_key_from_pem(&public_key)?,
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
    // TODO should be a custom error type
    pub fn to_jwt(&self, key: &Key) -> Result<String, Box<dyn Error>> {
        Ok(Token::new(
            jwt::Header {
                algorithm: AlgorithmType::Rs512,
                ..Default::default()
            },
            self,
        )
        .sign_with_key(&key.private)?
        .as_str()
        .to_string())
    }

    // TODO should be a custom error type, which includes things like expired
    pub fn from_jwt_and_validate(key: &Key, jwt: &str) -> Result<Self, Box<dyn Error>> {
        // TODO make sure to check that jwt is not expired, presumably verify does that?

        let token: Token<jwt::Header, Self, _> = jwt.verify_with_key(&key.public)?;
        trace!("jwt header: {:?}", token.header());
        trace!("jwt claims: {:?}", token.claims());

        Ok(Self {
            username: token.claims().username.to_string(),
        })
    }
}
