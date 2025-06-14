use std::fs::{self, OpenOptions};
use std::io::{Read, Write};
use std::sync::{Arc, Mutex};

#[derive(Clone)]
pub struct CachedMsg {
    pub dest: String,
    pub data: Vec<u8>,
}

pub struct Cache {
    pub cache: Arc<Mutex<Vec<CachedMsg>>>,
    path: String,
}

impl Cache {
    pub fn new(path: &str) -> Self {
        let mut cache_vec = Vec::new();
        if let Ok(entries) = fs::read_dir(path) {
            for entry in entries.flatten() {
                let mut file = OpenOptions::new().read(true).open(entry.path()).unwrap();
                let mut data = Vec::new();
                file.read_to_end(&mut data).unwrap();
                let dest = entry.file_name().to_string_lossy().into_owned();
                cache_vec.push(CachedMsg { dest, data });
            }
        }
        Self {
            cache: Arc::new(Mutex::new(cache_vec)),
            path: path.into(),
        }
    }

    pub fn put(&self, msg: &CachedMsg) -> std::io::Result<()> {
        let mut cache = self.cache.lock().unwrap();
        let filename = format!("{}/{}", self.path, msg.dest);
        let mut file = OpenOptions::new().create(true).write(true).open(&filename)?;
        file.write_all(&msg.data)?;
        cache.push(msg.clone());
        Ok(())
    }

    pub fn take_all(&self) -> Vec<CachedMsg> {
        let mut cache = self.cache.lock().unwrap();
        let all = cache.clone();
        cache.clear();
        for msg in &all {
            let filename = format!("{}/{}", self.path, msg.dest);
            let _ = fs::remove_file(&filename);
        }
        all
    }
}