use advanced_dtn_node::cache::{Cache, CachedMsg};
use tempfile::tempdir;

#[test]
fn test_cache_put_and_take() {
    let dir = tempdir().unwrap();
    let cache = Cache::new(dir.path().to_str().unwrap());
    let msg = CachedMsg {
        dest: "peer1".to_string(),
        data: vec![1, 2, 3],
    };
    cache.put(&msg).unwrap();

    let entries = cache.take_all();
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].dest, "peer1");
    assert_eq!(entries[0].data, vec![1, 2, 3]);
}