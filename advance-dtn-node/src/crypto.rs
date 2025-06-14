use std::fs::File;
use rsa::{
    RsaPrivateKey, RsaPublicKey, Pkcs1v15Sign, PaddingScheme,
    pkcs8::{EncodePublicKey, EncodePrivateKey, DecodePrivateKey, DecodePublicKey},
};
use rand::rngs::OsRng;
use sha2::{Sha256, Digest};
use aes_gcm::{Aes256Gcm, Key, Nonce};
use aes_gcm::aead::{Aead, KeyInit};

pub fn load_or_generate_key(path: &str) -> Result<RsaPrivateKey, Box<dyn std::error::Error>> {
    if let Ok(mut file) = File::open(path) {
        let mut buf = Vec::new();
        file.read_to_end(&mut buf)?;
        Ok(RsaPrivateKey::from_pkcs8_der(&buf)?)
    } else {
        let mut rng = OsRng;
        let key = RsaPrivateKey::new(&mut rng, 2048)?;
        let der = key.to_pkcs8_der()?;
        let mut file = File::create(path)?;
        file.write_all(&der)?;
        Ok(key)
    }
}

pub fn get_public_der(pub_key: &RsaPublicKey) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    Ok(pub_key.to_public_key_der()?.as_bytes().to_vec())
}

pub fn sign(data: &[u8], key: &RsaPrivateKey) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let mut hasher = Sha256::new();
    hasher.update(data);
    let hashed = hasher.finalize();
    Ok(key.sign(Pkcs1v15Sign::new::<Sha256>(), &hashed)?)
}

pub fn verify(data: &[u8], sig: &[u8], key: &RsaPublicKey) -> Result<bool, Box<dyn std::error::Error>> {
    let mut hasher = Sha256::new();
    hasher.update(data);
    let hashed = hasher.finalize();
    Ok(key.verify(Pkcs1v15Sign::new::<Sha256>(), &hashed, sig).is_ok())
}

pub fn encrypt_for(data: &[u8], pub_key: &RsaPublicKey) -> Result<(Vec<u8>, Vec<u8>, Vec<u8>), Box<dyn std::error::Error>> {
    let mut rng = OsRng;
    let key = Key::from_slice(&rand::random::<[u8; 32]>());
    let nonce = rand::random::<[u8; 12]>();
    let cipher = Aes256Gcm::new(key);
    let ct = cipher.encrypt(Nonce::from_slice(&nonce), data)?;
    // Encrypt key with RSA public key
    let enc_key = pub_key.encrypt(&mut rng, PaddingScheme::new_pkcs1v15_encrypt(), key)?;
    Ok((enc_key, nonce.to_vec(), ct))
}

pub fn decrypt_from(enc_key: &[u8], nonce: &[u8], ct: &[u8], priv_key: &RsaPrivateKey) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let key = priv_key.decrypt(PaddingScheme::new_pkcs1v15_encrypt(), enc_key)?;
    let cipher = Aes256Gcm::new(Key::from_slice(&key));
    Ok(cipher.decrypt(Nonce::from_slice(nonce), ct)?)
}