use radiobrowser::{RadioBrowserAPI, StationOrder};
use serde::{Deserialize, Serialize};
use strum::EnumString;

use crate::errors::Error;

#[derive(Debug)]
pub struct RadioApi {
    api: RadioBrowserAPI,
}

impl RadioApi {
    pub async fn new() -> Result<Self, Error> {
        Ok(Self {
            api: RadioBrowserAPI::new().await?,
        })
    }

    pub async fn get_stations(
        &self,
        params: Vec<SearchParam>,
    ) -> Result<Vec<super::RadioStation>, Error> {
        let mut builder = self
            .api
            .get_stations()
            .reverse(true)
            .limit(30.to_string())
            .order(Order::Votes.into());

        for param in params.into_iter() {
            tracing::info!(?param, "building search");
            match param {
                SearchParam::Name(name) => builder = builder.name(name),
                SearchParam::Language(language) => builder = builder.language(language),
                SearchParam::Country(country) => builder = builder.country(country),
                SearchParam::Tags(tags) => {
                    builder = builder.tag_list(tags.iter().map(|x| x.as_str()).collect())
                }
                SearchParam::Limit(limit) => builder = builder.limit(limit.to_string()),
                SearchParam::Reverse(reverse) => builder = builder.reverse(reverse),
                SearchParam::Order(order) => builder = builder.order(order.into()),
            }
        }

        Ok(builder
            .send()
            .await?
            .into_iter()
            .map(super::RadioStation::from)
            .collect())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, EnumString)]
pub enum Order {
    Name,
    Url,
    Homepage,
    Favicon,
    Tags,
    Country,
    State,
    Language,
    Votes,
    Codec,
    Bitrate,
    Lastcheckok,
    Lastchecktime,
    Clicktimestamp,
    Clicks,
    RecentTrend,
    Changetimestamp,
    Random,
}

impl From<Order> for StationOrder {
    fn from(value: Order) -> Self {
        match value {
            Order::Name => StationOrder::Name,
            Order::Url => StationOrder::Url,
            Order::Homepage => StationOrder::Homepage,
            Order::Favicon => StationOrder::Favicon,
            Order::Tags => StationOrder::Tags,
            Order::Country => StationOrder::Country,
            Order::State => StationOrder::State,
            Order::Language => StationOrder::Language,
            Order::Votes => StationOrder::Votes,
            Order::Codec => StationOrder::Codec,
            Order::Bitrate => StationOrder::Bitrate,
            Order::Lastcheckok => StationOrder::Lastcheckok,
            Order::Lastchecktime => StationOrder::Lastchecktime,
            Order::Clicktimestamp => StationOrder::Clicktimestamp,
            Order::Clicks => StationOrder::Clickcount,
            Order::RecentTrend => StationOrder::Clicktrend,
            Order::Changetimestamp => StationOrder::Changetimestamp,
            Order::Random => StationOrder::Random,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum SearchParam {
    Name(String),
    Country(String),
    Language(String),
    Tags(Vec<String>),
    Limit(usize),
    Reverse(bool),
    Order(Order),
}
