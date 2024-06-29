mod app;
mod errors;
mod models;
mod tui;

use std::io;

#[tokio::main]
async fn main() -> io::Result<()> {
    let mut terminal = tui::init()?;
    let mut app = app::App::new().await.expect("Failed to create app");
    let app_result = app.run(&mut terminal);

    tui::restore().expect("Failed to restore terminal");
    app_result.await
}
// async fn main() -> Result<()> {
//     // let api = RadioBrowserAPI::new().await.expect("Failed to create API");
//     // let stations = api
//     //     .get_stations()
//     //     .name("kexp")
//     //     .reverse(true)
//     //     .order(StationOrder::Clickcount)
//     //     .send()
//     //     .await
//     //     .expect("Failed to get stations");
//     //
//     // println!("Stations found: {}", stations.len());
//     //
//     // for s in stations {
//     //     println!("{:?}", s);
//     // }
//
//     stdout().execute(EnterAlternateScreen)?;
//     enable_raw_mode()?;
//     let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;
//     terminal.clear()?;
//
//     // TODO main loop
//     loop {
//         // TODO draw the UI
//         terminal.draw(|frame| {
//             let area = frame.size();
//             frame.render_widget(
//                 Paragraph::new("Hello Ratatouille! (press 'q' to quit)")
//                     .white()
//                     .on_blue(),
//                 area,
//             );
//         })?;
//         // TODO handle events
//         if event::poll(std::time::Duration::from_millis(16))? {
//             if let event::Event::Key(key) = event::read()? {
//                 if key.kind == KeyEventKind::Press && key.code == KeyCode::Char('q') {
//                     break;
//                 }
//             }
//         }
//     }
//
//     stdout().execute(LeaveAlternateScreen)?;
//     disable_raw_mode()?;
//     Ok(())
//
//     // println!("https://kexp-mp3-128.streamguys1.com/kexp128.mp3");
//     //
//     // let url = "https://kexp-mp3-128.streamguys1.com/kexp128.mp3";
//     //
//     // let mut station = RadioStation::new(url);
//     //
//     // let _guard = station.play().await.expect("Failed to play station");
//     //
//     // // station.sink.unwrap().sleep_until_end();
//     //
//     // tokio::select! {
//     //     _ = signal::ctrl_c() => {},
//     // }
// }
