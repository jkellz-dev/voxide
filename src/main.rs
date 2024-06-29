use radiobrowser::{RadioBrowserAPI, StationOrder};
use tokio::signal;
use voxide::models::RadioStation;

#[tokio::main]
async fn main() {
    let api = RadioBrowserAPI::new().await.expect("Failed to create API");
    let stations = api
        .get_stations()
        .name("kexp")
        .reverse(true)
        .order(StationOrder::Clickcount)
        .send()
        .await
        .expect("Failed to get stations");

    println!("Stations found: {}", stations.len());

    for s in stations {
        println!("{:?}", s);
    }

    println!("https://kexp-mp3-128.streamguys1.com/kexp128.mp3");

    let url = "https://kexp-mp3-128.streamguys1.com/kexp128.mp3";

    let mut station = RadioStation::new(url);

    let _guard = station.play().await.expect("Failed to play station");

    // station.sink.unwrap().sleep_until_end();

    tokio::select! {
        _ = signal::ctrl_c() => {},
    }
}
