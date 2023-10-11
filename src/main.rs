pub mod items;
pub mod page;

use dotenv::dotenv;
use page::Auction;
use rocket::{
    fs::{relative, FileServer},
    serde::json::Json,
    State,
};
use std::{
    collections::HashMap,
    env,
    sync::Arc,
    time::{Duration, Instant},
};
use tokio::time::sleep;

use crate::items::ItemsCache;

#[macro_use]
extern crate rocket;

impl GetCaseInsensitive<Vec<Auction>> for HashMap<String, Vec<Auction>> {
    fn get_case_insensitive(&self, key: &str) -> Vec<Auction> {
        match self.get(&key.to_lowercase()) {
            None => Vec::new(),
            Some(x) => x.to_vec(),
        }
    }
}
pub trait GetCaseInsensitive<T> {
    fn get_case_insensitive(&self, key: &str) -> T;
}

async fn update_cache_periodically(cache: Arc<ItemsCache>, update_interval: u64) {
    loop {
        println!("Updating cache");
        let start = Instant::now();
        cache.update().await;
        let duration = start.elapsed();
        println!("Updated cache in {:#?} seconds", duration);
        sleep(Duration::from_secs(update_interval)).await
    }
}

#[get("/items")]
async fn get_all_items(items: &State<Arc<ItemsCache>>) -> Json<Vec<String>> {
    let val: Vec<String> = items.map.lock().await.keys().cloned().collect();
    Json(val)
}

#[get("/search?<search>")]
async fn search_item(
    search: Option<String>,
    items: &State<Arc<ItemsCache>>,
) -> Option<Json<Vec<Auction>>> {
    match search {
        Some(item_name) => {
            let auctions = items.map.lock().await.get_case_insensitive(&item_name);
            Some(Json(auctions))
        },
        None => None,
    }
}

#[launch]
async fn launch() -> _ {
    dotenv().ok();
    let api_key: String = env::var("API_KEY").ok().unwrap();
    let cache = Arc::new(ItemsCache::new(api_key).await);
    let cache_clone = Arc::clone(&cache);
    tokio::spawn(async move {
        update_cache_periodically(cache_clone, 60).await;
    });
    rocket::build()
        .manage(cache)
        .mount("/", FileServer::from(relative!("static")))
        .mount("/", routes![get_all_items, search_item])
}
