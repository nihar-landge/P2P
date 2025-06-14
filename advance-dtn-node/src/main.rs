mod crypto;
mod message;
mod cache;
mod peers;
mod ble;
mod ble_transfer;

mod ffi;

use crate::crypto::*;
use crate::message::*;
use crate::cache::*;
use crate::peers::*;
use crate::ble::*;
use crate::ble_transfer::*;

use clap::{Parser, Subcommand};
use dtn7::client::{DtnClient, DtnConfig};
use rsa::{RsaPrivateKey, RsaPublicKey};
use std::time::{SystemTime, UNIX_EPOCH};
use std::error::Error;
use std::sync::{Arc, Mutex};

#[derive(Parser)]
#[command(name = "Advanced DTN Node")]
struct Cli {
    #[arg(long)]
    eid: String,
    #[arg(long, default_value = "id.pem")]
    key: String,
    #[arg(long, default_value = "cache")]
    cache_dir: String,
    #[arg(long, default_value = "127.0.0.1:3000")]
    addr: String,
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    SendAlert { dest: String, text: String, urgency: u8, encrypt: bool },
    Listen,
    AddPeer { eid: String, addr: String },
    ListPeers,
    ListCache,
}

fn now() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs()
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let cli = Cli::parse();

    let priv_key = load_or_generate_key(&cli.key)?;
    let pub_key = RsaPublicKey::from(&priv_key);
    let pub_key_der = get_public_der(&pub_key)?;

    let mut dtn = DtnClient::with_config(DtnConfig {
        eid: Some(cli.eid.clone()),
        ..Default::default()
    }).await?;

    let peers = PeerDirectory::new();
    peers.start_mdns(cli.eid.clone(), cli.addr.clone());
    let cache = Cache::new(&cli.cache_dir);

    // --- BLE Integration ---
    let peers_arc = Arc::clone(&peers.peers);
    let default_port = "3000";
    let ble_cb = Arc::new(Mutex::new(move |peer_eid: String| {
        let mut peers = peers_arc.lock().unwrap();
        if !peers.contains_key(&peer_eid) {
            peers.insert(peer_eid.clone(), format!("ble:{}", default_port));
            println!("[PEER] Added BLE-discovered peer {}: ble:{}", peer_eid, default_port);
        }
    }));
    let ble = BleDiscovery::new(cli.eid.clone(), ble_cb);
    ble.start();
    // --- End BLE Integration ---

    // --- BLE GATT Server for Bundle Transfer (stub) ---
    let cache_arc = Arc::clone(&cache.cache);
    let on_bundle = Arc::new(Mutex::new(move |bundle: Vec<u8>, sender_eid: String| {
        println!(
            "[BLE-TRANSFER] Received bundle from {} ({} bytes)",
            sender_eid,
            bundle.len()
        );
        // Optionally: cache the bundle, or process directly
        let mut cache = cache_arc.lock().unwrap();
        cache.push(CachedMsg { dest: sender_eid, data: bundle });
    }));
    start_ble_gatt_server(cli.eid.clone(), on_bundle.clone());
    // --- End BLE GATT Server ---

    match cli.command {
        Commands::SendAlert { dest, text, urgency, encrypt } => {
            let kind = DataKind::Alert { text, urgency };
            let timestamp = now();
            let plain = serde_json::to_vec(&(cli.eid.clone(), &kind, timestamp))?;
            let sig = sign(&plain, &priv_key)?;
            let mut env = Envelope {
                sender: cli.eid.clone(),
                sender_pub: pub_key_der.clone(),
                kind,
                timestamp,
                sig,
                encrypted: false,
                enc_key: None,
                nonce: None,
                ciphertext: None,
            };
            let mut payload = serde_json::to_vec(&env)?;
            if encrypt {
                if let Some(_addr) = peers.get(&dest) {
                    let (enc_key, nonce, ciphertext) = encrypt_for(&payload, &pub_key)?;
                    env.encrypted = true;
                    env.enc_key = Some(enc_key);
                    env.nonce = Some(nonce);
                    env.ciphertext = Some(ciphertext);
                    payload = serde_json::to_vec(&env)?;
                }
            }
            if let Some(addr) = peers.get(&dest) {
                if addr.starts_with("ble:") {
                    if let Err(e) = send_bundle_via_ble(addr.clone(), payload.clone()) {
                        println!("BLE bundle send failed: {e}");
                        cache.put(&CachedMsg {
                            dest: dest.clone(),
                            data: payload.clone(),
                        })?;
                    }
                } else {
                    if let Err(e) = dtn.send(&dest, payload.clone()).await {
                        println!("DTN send failed, caching: {e}");
                        cache.put(&CachedMsg { dest: dest.clone(), data: payload })?;
                    }
                }
            } else {
                println!("Unknown peer {}, caching", dest);
                cache.put(&CachedMsg { dest: dest.clone(), data: payload })?;
            }
        }
        Commands::Listen => {
            loop {
                for cached in cache.take_all() {
                    if let Some(addr) = peers.get(&cached.dest) {
                        if addr.starts_with("ble:") {
                            if let Err(e) = send_bundle_via_ble(addr.clone(), cached.data.clone()) {
                                println!("BLE bundle resend failed, keeping: {e}");
                                cache.put(&cached)?;
                            }
                        } else if let Err(e) = dtn.send(&cached.dest, cached.data.clone()).await {
                            println!("Cache resend failed, keeping: {e}");
                            cache.put(&cached)?;
                        }
                    } else {
                        cache.put(&cached)?; // Peer not known yet
                    }
                }
                let bundle = dtn.recv().await?;
                if let Ok(env) = serde_json::from_slice::<Envelope>(&bundle.data) {
                    let payload = if env.encrypted {
                        if let (Some(enc_key), Some(nonce), Some(ct)) = (env.enc_key.as_deref(), env.nonce.as_deref(), env.ciphertext.as_deref()) {
                            decrypt_from(enc_key, nonce, ct, &priv_key)?
                        } else {
                            println!("Missing encryption fields in envelope");
                            continue;
                        }
                    } else {
                        bundle.data.clone()
                    };
                    let env2: Envelope = serde_json::from_slice(&payload)?;
                    let plain = serde_json::to_vec(&(env2.sender.clone(), &env2.kind, env2.timestamp))?;
                    let sender_pub = RsaPublicKey::from_public_key_der(&env2.sender_pub)?;
                    let is_ok = verify(&plain, &env2.sig, &sender_pub)?;
                    if is_ok {
                        println!("[RECV] {:#?}", env2.kind);
                        let mut peers = peers.peers.lock().unwrap();
                        if let Some(addr) = peers.get_mut(&env2.sender) {
                            if addr.starts_with("ble:") {
                                *addr = "addr_learned_from_bundle".to_string();
                            }
                        }
                    } else {
                        println!("[!] Signature verification failed");
                    }
                } else {
                    println!("Unknown bundle format: {:?}", bundle.data);
                }
            }
        }
        Commands::AddPeer { eid, addr } => {
            peers.add(eid, addr);
            println!("Peer added.");
        }
        Commands::ListPeers => {
            for (eid, addr) in peers.all() {
                println!("{} -> {}", eid, addr);
            }
        }
        Commands::ListCache => {
            let entries = cache.take_all();
            if entries.is_empty() {
                println!("Cache is empty.");
            } else {
                for (i, msg) in entries.iter().enumerate() {
                    println!("[{}] Dest: {}, Data: {} bytes", i, msg.dest, msg.data.len());
                }
            }
            for msg in entries {
                cache.put(&msg)?;
            }
        }
    }
    Ok(())
}