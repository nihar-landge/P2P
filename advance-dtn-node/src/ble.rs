use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use btleplug::api::{Central, Manager as _, Peripheral as _, ScanFilter};
use btleplug::platform::Manager;

/// BLE-based peer discovery: discovers EIDs via BLE device name prefix 'DTN:'
pub struct BleDiscovery {
    pub eid: String,
    discovered: Arc<Mutex<Vec<String>>>,
    pub callback: Arc<Mutex<dyn Fn(String) + Send + Sync>>,
}

impl BleDiscovery {
    pub fn new(eid: String, callback: Arc<Mutex<dyn Fn(String) + Send + Sync>>) -> Self {
        Self {
            eid,
            discovered: Arc::new(Mutex::new(Vec::new())),
            callback,
        }
    }

    pub fn start(&self) {
        let eid = self.eid.clone();
        let discovered = Arc::clone(&self.discovered);
        let cb = Arc::clone(&self.callback);

        thread::spawn(move || {
            let manager = Manager::new().unwrap();
            let adapters = manager.adapters().unwrap();
            let central = adapters.into_iter().nth(0).unwrap();

            // Start scanning
            central.start_scan(ScanFilter::default()).unwrap();

            loop {
                for p in central.peripherals().unwrap() {
                    if let Ok(Some(props)) = p.properties() {
                        if let Some(name) = props.local_name {
                            if name.starts_with("DTN:") {
                                let peer_eid = name.trim_start_matches("DTN:").to_string();
                                if peer_eid != eid {
                                    let mut found = discovered.lock().unwrap();
                                    if !found.contains(&peer_eid) {
                                        found.push(peer_eid.clone());
                                        // Notify main logic
                                        let cb = cb.lock().unwrap();
                                        cb(peer_eid.clone());
                                        println!("[BLE] Discovered peer: {}", peer_eid);
                                    }
                                }
                            }
                        }
                    }
                }
                thread::sleep(Duration::from_secs(5));
            }
        });

        // (BLE advertising is platform-specific and not included for brevity)
        // You can use btleplug's Peripheral::start_advertising to advertise "DTN:<eid>"
    }
}