use aes_gcm::{Aes256Gcm, Key};
use base64::{Engine, prelude::BASE64_STANDARD};
use josekit::{
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
}

pub fn encrypt(claims: &UserClaims, key: &Jwk) -> anyhow::Result<String> {
    // Create the JWE header
    let mut jwe_header = JweHeader::new();
    jwe_header.set_token_type("JWT");
    jwe_header.set_algorithm("A256GCMKW");
    jwe_header.set_content_encryption("A256GCM");

    // Create the JWT payload
    let mut jwt_payload = JwtPayload::new();
    jwt_payload.set_claim("session_id", Some(claims.session_id.to_string().into()))?;
    jwt_payload.set_claim("user_id", Some(claims.user_id.to_string().into()))?;
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

pub fn decrypt(input: &[u8], key: &Jwk) -> anyhow::Result<UserClaims> {
    // Decrypt the input
    let decrypter = A256gcmkw.decrypter_from_jwk(key)?;
    let (payload, _) = jwt::decode_with_decrypter(input, &decrypter)?;

    // Create user claims
    Ok(UserClaims {
        session_id: Uuid::parse_str(
            payload
                .claim("session_id")
                .ok_or(anyhow::anyhow!("session_id required"))?
                .as_str()
                .ok_or(anyhow::anyhow!("session_id must be a UUID"))?,
        )?,
        user_id: Uuid::parse_str(
            payload
                .claim("user_id")
                .ok_or(anyhow::anyhow!("user_id required"))?
                .as_str()
                .ok_or(anyhow::anyhow!("user_id must be a UUID"))?,
        )?,
        user_key: *Key::<Aes256Gcm>::from_slice(
            &BASE64_STANDARD.decode(
                payload
                    .claim("user_key")
                    .ok_or(anyhow::anyhow!("user_key required"))?
                    .as_str()
                    .ok_or(anyhow::anyhow!("user_key must be a string value"))?,
            )?,
        ),
    })
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
