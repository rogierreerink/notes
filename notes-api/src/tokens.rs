use aes_gcm::{Aes256Gcm, Key};
use base64::{Engine, prelude::BASE64_STANDARD};
use josekit::{
    JoseError,
    jwe::{JweHeader, alg::aesgcmkw::AesgcmkwJweAlgorithm::A256gcmkw},
    jwk::Jwk,
    jwt::{self, JwtPayload},
};
use uuid::Uuid;

#[derive(Debug, PartialEq)]
pub struct UserClaims {
    session_id: Uuid,
    user_id: Uuid,
    user_key: Key<Aes256Gcm>,
}

impl UserClaims {
    pub fn new(session_id: Uuid, user_id: Uuid, user_key: Key<Aes256Gcm>) -> Self {
        Self {
            session_id,
            user_id,
            user_key,
        }
    }

    pub fn session_id(&self) -> &Uuid {
        &self.session_id
    }

    pub fn user_id(&self) -> &Uuid {
        &self.user_id
    }

    pub fn user_key(&self) -> &Key<Aes256Gcm> {
        &self.user_key
    }
}

pub fn encrypt(claims: &UserClaims, key: &Jwk) -> anyhow::Result<String> {
    // Create the JWE header
    let mut jwe_header = JweHeader::new();
    jwe_header.set_token_type("JWT");
    jwe_header.set_algorithm("A256GCMKW");
    jwe_header.set_content_encryption("A256GCM");
    jwe_header.set_claim("session_id", Some(claims.session_id.to_string().into()))?;
    jwe_header.set_claim("user_id", Some(claims.user_id.to_string().into()))?;

    // Create the JWT payload
    let mut jwt_payload = JwtPayload::new();
    jwt_payload.set_claim(
        "user_key",
        Some(BASE64_STANDARD.encode(&claims.user_key).into()),
    )?;

    // Encrypt the JWT
    let encrypter = A256gcmkw.encrypter_from_jwk(key)?;
    Ok(jwt::encode_with_encrypter(
        &jwt_payload,
        &jwe_header,
        &encrypter,
    )?)
}

pub fn decrypt(input: &[u8], key: &Jwk) -> Result<UserClaims, TokenDecryptionError> {
    // Decrypt the input
    let decrypter = A256gcmkw
        .decrypter_from_jwk(key)
        .map_err(|_| TokenDecryptionError::Internal)?;
    let (payload, header) = jwt::decode_with_decrypter(input, &decrypter).map_err(|e| match e {
        JoseError::InvalidJweFormat(_) => TokenDecryptionError::InvalidKey,
        _ => TokenDecryptionError::Internal,
    })?;

    let header_claims = header.claims_set();
    let payload_claims = payload.claims_set();

    // Parse user claims
    let session_id = Uuid::parse_str(get_required_claim(header_claims, "session_id")?)
        .map_err(|e| TokenDecryptionError::InvalidClaim(anyhow::anyhow!("session_id: {}", e)))?;
    let user_id = Uuid::parse_str(get_required_claim(header_claims, "user_id")?)
        .map_err(|e| TokenDecryptionError::InvalidClaim(anyhow::anyhow!("user_id: {}", e)))?;
    let user_key = *Key::<Aes256Gcm>::from_slice(
        &BASE64_STANDARD
            .decode(get_required_claim(payload_claims, "user_key")?)
            .map_err(|e| TokenDecryptionError::InvalidClaim(anyhow::anyhow!("user_key: {}", e)))?,
    );

    Ok(UserClaims {
        session_id,
        user_id,
        user_key,
    })
}

fn get_required_claim<'a>(
    claims: &'a josekit::Map<String, josekit::Value>,
    name: &str,
) -> Result<&'a str, TokenDecryptionError> {
    claims
        .get(name)
        .and_then(|claim| claim.as_str())
        .ok_or(TokenDecryptionError::InvalidClaim(anyhow::anyhow!(
            format!("`{}` claim must be set", name)
        )))
}

#[derive(thiserror::Error, Debug)]
pub enum TokenDecryptionError {
    #[error("invalid key")]
    InvalidKey,

    #[error("invalid claim: {0}")]
    InvalidClaim(#[source] anyhow::Error),

    #[error("internal error")]
    Internal,
}

#[cfg(test)]
mod tests {
    use aes_gcm::{Aes256Gcm, KeyInit, aead::OsRng};
    use josekit::jwk::Jwk;
    use uuid::Uuid;

    use crate::tokens::{UserClaims, decrypt, encrypt};

    #[test]
    fn encrypt_decrypt_user_claims() {
        let jwk = Jwk::generate_oct_key(32).expect("failed to generate jwk");

        let user_claims = UserClaims::new(
            Uuid::new_v4(),
            Uuid::new_v4(),
            Aes256Gcm::generate_key(&mut OsRng),
        );

        let user_claims_encrypted =
            encrypt(&user_claims, &jwk).expect("failed to encrypt user claims");
        let user_claims_decrypted =
            decrypt(user_claims_encrypted.as_bytes(), &jwk).expect("failed to decrypt user claims");

        assert_eq!(user_claims, user_claims_decrypted);
    }
}
