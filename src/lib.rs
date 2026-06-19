use std::collections::HashMap;
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone)]
pub struct Record {
    pub key: String,
    pub value: f64,
    pub timestamp: u64,
}

pub struct Store {
    data: Arc<Mutex<HashMap<String, Vec<Record>>>>,
}

impl Store {
    pub fn new() -> Self {
        Self { data: Arc::new(Mutex::new(HashMap::new())) }
    }

    pub fn insert(&self, key: &str, value: f64, timestamp: u64) {
        let mut data = self.data.lock().unwrap();
        data.entry(key.to_string()).or_insert_with(Vec::new).push(Record {
            key: key.to_string(),
            value,
            timestamp,
        });
    }

    pub fn get(&self, key: &str) -> Vec<Record> {
        let data = self.data.lock().unwrap();
        data.get(key).cloned().unwrap_or_default()
    }

    pub fn count(&self) -> usize {
        let data = self.data.lock().unwrap();
        data.values().map(|v| v.len()).sum()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_insert_and_get() {
        let store = Store::new();
        store.insert("cpu", 75.5, 1000);
        store.insert("cpu", 80.0, 2000);
        let records = store.get("cpu");
        assert_eq!(records.len(), 2);
        assert_eq!(store.count(), 2);
    }

    #[test]
    fn test_empty_store() {
        let store = Store::new();
        assert_eq!(store.count(), 0);
        assert!(store.get("missing").is_empty());
    }
}
