use crate::{config::Service as ConfigService, errors::Error};
use chrono::{DateTime, Duration, Utc};
use jwt::{AlgorithmType, PKeyWithDigest, SignWithKey, Token, VerifyWithKey};
use openssl::{
    hash::MessageDigest,
    pkey::{PKey, Private, Public},
    rsa::Rsa,
};
use serde::{Deserialize, Serialize};

pub struct Service {
    private: PKeyWithDigest<Private>,
    public: PKeyWithDigest<Public>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    pub username: String,
    #[serde(rename = "iat")]
    pub issued_at: DateTime<Utc>,
    #[serde(rename = "exp")]
    pub expires_at: DateTime<Utc>,
}

impl Claims {
    pub fn new(username: &str) -> Claims {
        let now = Utc::now();
        let expires = now + Duration::minutes(15);
        Self {
            username: username.to_string(),
            issued_at: now,
            expires_at: expires,
        }
    }
}

impl Service {
    pub async fn new(config_service: &ConfigService) -> Result<Self, Box<dyn std::error::Error>> {
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

    pub fn create_jwt(&self, claims: &Claims) -> Result<String, Box<dyn std::error::Error>> {
        Ok(Token::new(
            jwt::Header {
                algorithm: AlgorithmType::Rs512,
                ..Default::default()
            },
            claims,
        )
        .sign_with_key(&self.private)?
        .as_str()
        .to_string())
    }

    pub fn validate(&self, jwt: &str) -> Result<Claims, Error> {
        let token: Token<jwt::Header, Claims, _> = match jwt.verify_with_key(&self.public) {
            Ok(token) => token,
            Err(e) => {
                debug!("jwt validation failed: {e:?}");
                Err(Error::Unauthorized)?
            }
        };
        trace!("jwt header: {:?}", token.header());
        trace!("jwt claims: {:?}", token.claims());

        if token.claims().expires_at < Utc::now() {
            debug!("jwt is expired");
            Err(Error::Unauthorized)
        } else {
            trace!("jwt is still valid");
            Ok(token.claims().clone())
        }
    }
}
