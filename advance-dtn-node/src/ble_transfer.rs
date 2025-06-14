use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use btleplug::platform::{Manager, Peripheral as PlatformPeripheral, Adapter};
use uuid::Uuid;

/// The UUIDs for your custom BLE GATT service and characteristic.
/// Change these to your own unique values!
const BUNDLE_SERVICE_UUID: Uuid = Uuid::from_u128(0x12345678_1234_5678_1234_567812345678);
const BUNDLE_CHAR_UUID: Uuid = Uuid::from_u128(0x87654321_4321_8765_4321_876543218765);

/// Start BLE GATT server for receiving bundles
pub fn start_ble_gatt_server<F>(eid: String, on_bundle: Arc<Mutex<F>>)
where
    F: Fn(Vec<u8>, String) + Send + Sync + 'static,
{
    // NOTE: btleplug does NOT currently support acting as a GATT server (as of mid-2024)
    // This is pseudocode/architecture. On Linux, you can use BlueZ's dbus API (via zbus or bluez crate) for server role.
    // For now, print message to show where GATT server logic would be.
    println!("[BLE-TRANSFER] BLE GATT server not implemented in btleplug; use bluez crate or platform-specific code.");
}

/// BLE GATT client: connect to peer, send bundle
pub fn send_bundle_via_ble(
    peer_addr: String,
    bundle: Vec<u8>,
) -> Result<(), String> {
    let manager = Manager::new().map_err(|e| e.to_string())?;
    let adapters = manager.adapters().map_err(|e| e.to_string())?;
    let central = adapters.into_iter().nth(0).ok_or("No BLE adapter found")?;

    // Scan for the peer device by address (or EID in name)
    central.start_scan(ScanFilter::default()).map_err(|e| e.to_string())?;
    thread::sleep(Duration::from_secs(3)); // Allow scan to populate

    let peripherals = central.peripherals().map_err(|e| e.to_string())?;
    let peer = peripherals
        .into_iter()
        .find(|p| {
            if let Ok(Some(props)) = p.properties() {
                // Match by local_name (e.g., "DTN:<eid>") or address if available
                if let Some(name) = props.local_name {
                    name.contains(&peer_addr)
                } else {
                    false
                }
            } else {
                false
            }
        })
        .ok_or("Peer not found")?;

    peer.connect().map_err(|e| e.to_string())?;
    peer.discover_services().map_err(|e| e.to_string())?;

    // Find our custom service/characteristic
    let chars = peer.characteristics();
    let bundle_char = chars.iter().find(|c| c.uuid == BUNDLE_CHAR_UUID)
        .ok_or("Bundle characteristic not found")?;

    // BLE Write Max is ~512B; chunk if necessary
    let chunk_size = 200;
    for chunk in bundle.chunks(chunk_size) {
        peer.write(&bundle_char, chunk, WriteType::WithResponse)
            .map_err(|e| e.to_string())?;
        thread::sleep(Duration::from_millis(20));
    }
    peer.disconnect().ok();
    Ok(())
}