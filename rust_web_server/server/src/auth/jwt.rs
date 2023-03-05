use crate::config::Service as ConfigService;
use jwt::{AlgorithmType, PKeyWithDigest, SignWithKey, Token, VerifyWithKey};
use openssl::{
    hash::MessageDigest,
    pkey::{PKey, Private, Public},
    rsa::Rsa,
};
use serde::{Deserialize, Serialize};
use std::error::Error;

pub struct Service {
    private: PKeyWithDigest<Private>,
    public: PKeyWithDigest<Public>,
}

impl Service {
    pub async fn new(config_service: &ConfigService) -> Result<Self, Box<dyn Error>> {
        const PUBLIC_KEY_NAME: &str = "jwt_public_key";
        const PRIVATE_KEY_NAME: &str = "jwt_private_key";
        let public_key = config_service.get(PUBLIC_KEY_NAME).await?;
        let private_key = config_service.get(PRIVATE_KEY_NAME).await?;
        let digest = MessageDigest::sha512();
        Ok(match (public_key, private_key) {
            // already exists
            (Some(public_key), Some(private_key)) => {
                trace!("found JWT key in database");
                trace!("JWT public key\n{}", public_key.value);
                trace!("JWT private key\n{}", private_key.value);

                let public_key_pkey = PKey::public_key_from_pem(public_key.value.as_bytes())?;
                let private_key_pkey = PKey::private_key_from_pem(private_key.value.as_bytes())?;

                Self {
                    private: PKeyWithDigest {
                        digest,
                        key: private_key_pkey,
                    },
                    public: PKeyWithDigest {
                        digest,
                        key: public_key_pkey,
                    },
                }
            }

            // not yet
            (None, None) => {
                trace!("creating new JWT key");
                let rsa = Rsa::generate(4096)?;
                let private_key_pkey = PKey::from_rsa(rsa)?;

                let public_key = private_key_pkey.public_key_to_pem()?;
                let public_key_str = std::str::from_utf8(public_key.as_slice())?;
                trace!("JWT public key\n{public_key_str}");

                let private_key = private_key_pkey.private_key_to_pem_pkcs8()?;
                let private_key_str = std::str::from_utf8(private_key.as_slice())?;
                trace!("JWT private key\n{private_key_str}");

                let result = Self {
                    private: PKeyWithDigest {
                        digest,
                        key: private_key_pkey,
                    },
                    public: PKeyWithDigest {
                        digest,
                        key: PKey::public_key_from_pem(&public_key)?,
                    },
                };

                config_service.set(PUBLIC_KEY_NAME, public_key_str).await?;
                config_service
                    .set(PRIVATE_KEY_NAME, private_key_str)
                    .await?;
                trace!("updated db with new jwt key");

                result
            }

            // error
            (Some(_), None) => Err(format!("mismatched keys, missing {}", PRIVATE_KEY_NAME))?,
            (None, Some(_)) => Err(format!("mismatched keys, missing {}", PUBLIC_KEY_NAME))?,
        })
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub username: String,
}

impl Claims {
    // TODO should be a method on JwtService
    // TODO should expire
    // TODO should be a custom error type
    pub fn to_jwt(&self, service: &Service) -> Result<String, Box<dyn Error>> {
        Ok(Token::new(
            jwt::Header {
                algorithm: AlgorithmType::Rs512,
                ..Default::default()
            },
            self,
        )
        .sign_with_key(&service.private)?
        .as_str()
        .to_string())
    }

    // TODO should be a method on JwtService
    // TODO should be a custom error type, which includes things like expired
    pub fn from_jwt_and_validate(service: &Service, jwt: &str) -> Result<Self, Box<dyn Error>> {
        // TODO make sure to check that jwt is not expired, presumably verify does that?

        let token: Token<jwt::Header, Self, _> = jwt.verify_with_key(&service.public)?;
        trace!("jwt header: {:?}", token.header());
        trace!("jwt claims: {:?}", token.claims());

        Ok(Self {
            username: token.claims().username.to_string(),
        })
    }
}
