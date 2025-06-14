use std::collections::HashMap;

pub struct PeerDirectory {
    peers: HashMap<String, String>,
}

impl PeerDirectory {
    pub fn new() -> Self {
        PeerDirectory {
            peers: HashMap::new(),
        }
    }

    pub fn add(&mut self, eid: String, addr: String) {
        self.peers.insert(eid, addr);
    }

    pub fn get(&self, eid: &str) -> Option<String> {
        self.peers.get(eid).cloned()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_and_get_peer() {
        let mut dir = PeerDirectory::new();
        dir.add("eid1".to_string(), "addr1".to_string());
        assert_eq!(dir.get("eid1"), Some("addr1".to_string()));
    }
}