#![feature(decl_macro)]
pub mod page;

use dotenv::dotenv;
use page::{Auction, Page};
use rocket::{State, fs::{FileServer,relative}};
use std::{collections::HashMap, env};

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
fn items(item_auction_map: &State<HashMap<String, Vec<Auction>>>) -> String{
   let val : Vec<String> =  item_auction_map.keys().cloned().collect();
   val.join("<br>")
}

#[launch]
async fn launch() -> _ {
    dotenv().ok();
    let api_key: String = env::var("API_KEY").ok().unwrap();
    let item_auction_map = Page::get_map(&api_key).await.ok().unwrap();
    rocket::build()
        .manage(item_auction_map)
        .mount("/", FileServer::from(relative!("static")))
        .mount("/", routes![items])
}
