use std::{ops::Deref, sync::Arc};

use dashmap::DashMap;

use crate::resp::RespFrame;

#[derive(Debug, Clone, Default)]
pub struct Backend(Arc<BackendInner>);

#[derive(Debug, Default)]
pub struct BackendInner {
    map: DashMap<String, RespFrame>,
    hmap: DashMap<String, DashMap<String, RespFrame>>,
}
impl Deref for Backend {
    type Target = BackendInner;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl Backend {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn set(&self, key: String, value: RespFrame) {
        self.map.insert(key, value);
    }
    pub fn get(&self, key: &str) -> Option<RespFrame> {
        self.map.get(key).map(|v| v.clone())
    }
    pub fn hset(&self, key: String, field: String, value: RespFrame) {
        self.hmap.entry(key).or_default().insert(field, value);
    }
    pub fn hget(&self, key: &str, field: &str) -> Option<RespFrame> {
        self.hmap.get(key)?.get(field).map(|v| v.clone())
    }
    pub fn hgetall(&self, key: &str) -> Option<DashMap<String, RespFrame>> {
        self.hmap.get(key).map(|v| v.clone())
    }
}
