pub mod items;
pub mod page;

use dotenv::dotenv;
use page::Auction;
use rocket::{
    fs::{relative, FileServer},
    serde::json::Json,
    State,
};
use std::{collections::HashMap, env};

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

#[get("/items")]
fn get_all_items(items: &State<ItemsCache>) -> Json<Vec<String>> {
    let val: Vec<String> = items.map.lock().unwrap().keys().cloned().collect();
    Json(val)
}

#[get("/search?<search>")]
fn search_item(search: Option<String>, items: &State<ItemsCache>) -> Json<Vec<Auction>> {
    match search {
        Some(x) => {
            let auctions = items.map.lock().ok().unwrap().get_case_insensitive(&x);
            Json(auctions)
        }
        None => Json(Vec::new()),
    }
}

#[launch]
async fn launch() -> _ {
    dotenv().ok();
    let api_key: String = env::var("API_KEY").ok().unwrap();
    let items = ItemsCache::new(api_key).await;
    rocket::build()
        .manage(items)
        .mount("/", FileServer::from(relative!("static")))
        .mount("/", routes![get_all_items, search_item])
}
