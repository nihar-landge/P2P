use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug)]
pub enum DataKind {
    Alert { text: String, urgency: u8 },
    // Extend with other message types
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Envelope {
    pub sender: String,
    pub sender_pub: Vec<u8>,
    pub kind: DataKind,
    pub timestamp: u64,
    pub sig: Vec<u8>,
    pub encrypted: bool,
    pub enc_key: Option<Vec<u8>>,
    pub nonce: Option<Vec<u8>>,
    pub ciphertext: Option<Vec<u8>>,
}