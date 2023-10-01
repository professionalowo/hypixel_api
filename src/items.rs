use std::{collections::HashMap, sync::Mutex, pin::{Pin, pin}};
use rocket::{fairing::{Fairing, Info, Kind}, Data, Request, http::Method};

use crate::page::{Auction, Page};

pub struct ItemsCache {
    api_key: String,
    pub map: Mutex<HashMap<String, Vec<Auction>>>,
}

impl ItemsCache {
    pub async fn new(api_key: String) -> Self {
        let auctions = Page::get_map(&api_key).await.ok().unwrap();
        Self {
            map: auctions.into(),
            api_key,
        }
    }
    pub async fn update(&self) -> () {
        let new_map = Page::get_map(&self.api_key).await.ok().unwrap();
        let mut state = self.map.lock().expect("could not lock map");
        *state = new_map;
    }
}
