use core::fmt;
use exitfailure::ExitFailure;
use reqwest::Url;
use serde_derive::{Deserialize, Serialize};
use std::{cmp::Ordering, collections::HashMap};

#[derive(Serialize, Deserialize, Debug)]
#[allow(non_snake_case)]
pub struct Page {
    pub success: bool,
    pub page: u8,
    pub totalPages: u8,
    pub totalAuctions: u32,
    pub lastUpdated: u64,
    pub auctions: Option<Vec<Auction>>,
}

impl Page {
    async fn get_page(page_index: u8, api_key: String) -> Result<Self, ExitFailure> {
        let url = format!(
            "https://api.hypixel.net/skyblock/auctions?page={}&ApiKey={}",
            page_index, api_key
        );

        let url = Url::parse(&url)?;
        let res = reqwest::get(url).await?.json::<Page>().await?;

        Ok(res)
    }

    async fn get_number_of_pages(api_key: &str) -> Result<u8, ExitFailure> {
        let page = Self::get_page(0, api_key.into()).await?;
        Ok(page.totalPages)
    }
    async fn get_all_pages_parallel(api_key: &str) -> Result<Vec<Self>, ExitFailure> {
        let number_of_pages = Self::get_number_of_pages(api_key).await?;
        let mut tasks = Vec::with_capacity(number_of_pages.into());
        for i in 0..number_of_pages {
            tasks.push(tokio::spawn(Self::get_page(i, api_key.into())));
        }
        let mut pages = Vec::with_capacity(number_of_pages.into());
        for task in tasks {
            pages.push(task.await?.unwrap());
        }
        Ok(pages)
    }

    fn map_page_vec_to_hashmap(
        pages: &Vec<Self>,
    ) -> Result<HashMap<String, Vec<Auction>>, ExitFailure> {
        let mut map = HashMap::new();
        for page in pages {
            let auctions = page.auctions.as_ref().unwrap();
            for auction in auctions {
                let auction_clone = auction.clone();
                let item_name_clone = auction.item_name.clone().to_lowercase();
                map.entry(item_name_clone)
                    .or_insert(Vec::new())
                    .push(auction_clone);
            }
        }
        Ok(map)
    }

    pub async fn get_map(api_key: &str) -> Result<HashMap<String, Vec<Auction>>, ExitFailure> {
        let pages = Self::get_all_pages_parallel(api_key).await?;
        let map = Self::map_page_vec_to_hashmap(&pages).ok().unwrap();
        Ok(map)
    }
}

impl fmt::Display for Page {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Success: {}\nPage: {}\nTotalPages: {}\n Total Auctions:{}\n Auctions:{:#?}\n",
            self.success,
            self.page,
            self.totalPages,
            self.totalAuctions,
            self.auctions.as_ref().unwrap()
        )
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Auction {
    pub uuid: String,
    pub auctioneer: String,
    pub profile_id: String,
    pub coop: Vec<String>,
    pub start: u64,
    pub end: u64,
    pub item_name: String,
    pub item_lore: String,
    pub extra: String,
    pub category: String,
    pub tier: String,
    pub starting_bid: u64,
    pub item_bytes: String,
    pub claimed: bool,
    pub highest_bid_amount: u64,
    pub bids: Option<Vec<Bid>>,
    pub bin: bool,
}

impl fmt::Display for Auction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "name: {}, max_bid: {}",
            self.item_name, self.highest_bid_amount
        )
    }
}

#[derive(Serialize, Deserialize, Debug, Eq, Clone)]
pub struct Bid {
    pub auction_id: String,
    pub bidder: String,
    pub profile_id: String,
    pub amount: u64,
    pub timestamp: u64,
}

impl Ord for Bid {
    fn cmp(&self, other: &Self) -> Ordering {
        self.amount.cmp(&other.amount)
    }
}

impl PartialOrd for Bid {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for Bid {
    fn eq(&self, other: &Self) -> bool {
        self.amount == other.amount
    }
}
