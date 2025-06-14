use advanced_dtn_node::peers::PeerDirectory;

#[test]
fn test_add_and_get_peer() {
    let dir = PeerDirectory::new();
    dir.add("eid1".to_string(), "addr1".to_string());
    assert_eq!(dir.get("eid1"), Some("addr1".to_string()));
}

#[test]
fn test_all_peers() {
    let dir = PeerDirectory::new();
    dir.add("eid2".to_string(), "addr2".to_string());
    dir.add("eid3".to_string(), "addr3".to_string());
    let all = dir.all();
    assert!(all.contains(&("eid2".to_string(), "addr2".to_string())));
    assert!(all.contains(&("eid3".to_string(), "addr3".to_string())));
}