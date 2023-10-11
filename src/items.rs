use std::collections::HashMap;

use tokio::sync::Mutex;

use crate::page::{Auction, Page};

pub struct ItemsCache {
    api_key: String,
    pub map: Mutex<HashMap<String, Vec<Auction>>>,
}

impl ItemsCache {
    pub async fn new(api_key: String) -> Self {
        let api_key_clone = api_key.clone();
        let auctions = Page::get_map(&api_key).await.ok().unwrap();

        Self {
            map: auctions.into(),
            api_key: api_key_clone,
        }
    }
    pub async fn update(&self) -> () {
        let new_map = Page::get_map(&self.api_key)
            .await
            .ok()
            .unwrap();
        let mut cache_map = self.map.lock().await;
        *cache_map = new_map;
    }
}
