use advanced_dtn_node::crypto::{load_or_generate_key, sign, verify};
use rsa::RsaPublicKey;

#[test]
fn test_sign_and_verify() {
    let key = load_or_generate_key("test_id.pem").unwrap();
    let pub_key = RsaPublicKey::from(&key);
    let data = b"hello world";
    let sig = sign(data, &key).unwrap();
    let ok = verify(data, &sig, &pub_key).unwrap();
    assert!(ok);
    let _ = std::fs::remove_file("test_id.pem");
}