use radiobrowser::{RadioBrowserAPI, StationOrder};

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

    pub async fn get_stations(&self) -> Result<Vec<super::RadioStation>, Error> {
        Ok(self
            .api
            .get_stations()
            .name("kexp")
            .reverse(true)
            .order(StationOrder::Clickcount)
            .send()
            .await?
            .into_iter()
            .map(super::RadioStation::from)
            .collect())
    }
}
