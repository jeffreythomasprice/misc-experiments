use chrono::{TimeDelta, Utc};
use hmac::{Hmac, Mac};
use jwt::{Claims, RegisteredClaims, SignWithKey};
use sha2::Sha256;

use crate::db::models::User;
use anyhow::{anyhow, Result};

pub fn create_token(user: &User) -> Result<String> {
    let exp = Utc::now()
        .checked_add_signed(TimeDelta::minutes(5))
        .ok_or(anyhow!("failed to calculate expiration timestamp"))?;
    let mut claims = Claims::new(RegisteredClaims {
        issuer: None,
        subject: None,
        audience: None,
        expiration: Some(exp.timestamp() as u64),
        not_before: None,
        issued_at: None,
        json_web_token_id: None,
    });
    claims
        .private
        .insert("username".to_owned(), user.username.clone().into());
    // TODO static? lazy?
    let key: Hmac<Sha256> = Hmac::new_from_slice(b"secret")?;
    Ok(claims.sign_with_key(&key)?)
}
