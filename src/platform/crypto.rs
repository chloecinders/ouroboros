use aes_gcm::aead::{Aead, AeadCore, KeyInit, OsRng};
use aes_gcm::{Aes256Gcm, Key, Nonce};

pub type Secret = [u8; 32];

pub fn generate() -> Secret {
    Aes256Gcm::generate_key(OsRng).into()
}

pub fn encrypt(key: &Secret, plaintext: &str) -> Option<Vec<u8>> {
    let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(key));
    let nonce = Aes256Gcm::generate_nonce(&mut OsRng);
    let ciphertext = cipher.encrypt(&nonce, plaintext.as_bytes()).ok()?;
    let mut combined = nonce.to_vec();

    combined.extend_from_slice(&ciphertext);

    Some(combined)
}

pub fn decrypt(key: &Secret, ciphertext: &[u8]) -> Option<String> {
    if ciphertext.len() <= 12 {
        return None;
    }

    let (nonce, body) = ciphertext.split_at(12);
    let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(key));

    String::from_utf8(cipher.decrypt(Nonce::from_slice(nonce), body).ok()?).ok()
}
