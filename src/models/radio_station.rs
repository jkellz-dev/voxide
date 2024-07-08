use std::{
    io::Write,
    sync::{mpsc::RecvError, Arc},
    thread,
    time::Duration,
};

use futures::FutureExt;
use radiobrowser::ApiStation;
use ratatui::{
    prelude::*,
    style::{palette::tailwind, Color},
    text::Line,
    widgets::*,
};
use reqwest::header;
use rodio::{Decoder, OutputStream, Sink};
use serde::{Deserialize, Serialize};
use tokio::sync::{broadcast, mpsc, Mutex};

use crate::errors::Error;

use super::audio_stream::AudioStream;

const TODO_HEADER_BG: Color = tailwind::BLUE.c950;
const NORMAL_ROW_COLOR: Color = tailwind::SLATE.c950;
const ALT_ROW_COLOR: Color = tailwind::SLATE.c900;
const SELECTED_STYLE_FG: Color = tailwind::BLUE.c300;
const TEXT_COLOR: Color = tailwind::SLATE.c200;
const COMPLETED_TEXT_COLOR: Color = tailwind::GREEN.c500;

pub struct State {
    output_guard: Arc<Mutex<OutputStream>>,
    sink: Arc<Mutex<Sink>>,
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RadioStation {
    pub name: String,
    pub stationuuid: String,
    pub url: String,
    pub codec: String,
    pub bitrate: u32,
    pub homepage: String,
    pub tags: String,
    pub countrycode: String,
    pub languagecodes: Option<String>,
    pub votes: i32,
}

impl RadioStation {
    pub fn new<T: ToString>(url: T, stationuuid: T, name: T) -> Self {
        Self {
            name: name.to_string(),
            stationuuid: stationuuid.to_string(),
            url: url.to_string(),
            ..Default::default()
        }
    }
    pub async fn play(
        &mut self,
        mut download_shutdown_rx: broadcast::Receiver<()>,
        mut play_shutdown_rx: broadcast::Receiver<()>,
        initial_volume: f32,
        mut volume_rx: broadcast::Receiver<f32>,
        mut volume_shutdown_rx: broadcast::Receiver<()>,
    ) -> Result<(), Error> {
        tracing::info!(station = ?self, "playing");
        let client = reqwest::Client::new();
        let mut response = client
            .get(&self.url)
            .header(header::CONNECTION, "keep-alive")
            .send()
            .await?;

        tracing::debug!(?response, "got response");

        if response.status() != 200 {
            tracing::error!(?response, "failed to get stream");
            return Err(Error::Http(response.status()));
        }

        let audio_stream = AudioStream::new();

        let buf = audio_stream.get_buf();

        tracing::info!("spawning chunker");
        let handle = tokio::spawn(async move {
            tracing::info!("getting chunks...");

            loop {
                tokio::select! {
                    chunk = response.chunk() => {
                        match chunk {
                            Ok(chunk) => {
                                if let Some(chunk) = chunk {
                                    tracing::trace!("got chunk: {}", chunk.len());
                                    let mut guard = buf.lock().expect("failed to lock buffer");
                                    let result = guard.write(chunk.as_ref());
                                    match result {
                                        Ok(n) => tracing::trace!(bytes=?n, "pushed chunk"),
                                        Err(e) => tracing::error!(error=?e, "failed to get chunk"),
                                    }
                                }
                            }
                            Err(e) => {
                                tracing::error!("error {:?}", e);
                                continue;
                            }
                        }
                    }
                    _ = download_shutdown_rx.recv() => {
                        tracing::info!("chunker shutting down");
                        break;
                    }
                }
            }
        });

        tracing::debug!("waiting for chunks");

        while audio_stream.len()? < 1024 * 10 {
            tokio::task::yield_now().await;
        }

        tracing::info!("got enough chunks to start");

        // Spin off the stream handling to a task that allows blocking. This will then not use the
        // Tokio thread pool, but instead use a CPU managed thread.
        tokio::task::spawn_blocking(move || {
            // This is running on a thread where blocking is fine.
            tracing::info!("streaming task spawned");

            let (stream, stream_handle) = OutputStream::try_default().unwrap();
            let sink = Sink::try_new(&stream_handle).unwrap();

            tracing::debug!("setting up decoder");
            let decoder = Decoder::new_mp3(audio_stream).unwrap();
            sink.append(decoder);
            sink.set_volume(initial_volume);

            tokio::task::spawn(async move {
                loop {
                    tokio::select! {
                        vol = volume_rx.recv() => {
                            if let Ok(volume) = vol {
                                sink.set_volume(volume);
                            }
                        },
                        _ = volume_shutdown_rx.recv() => {
                            tracing::info!("Shutting down volume thread");
                            break;
                        }
                    }
                }
            });

            tracing::info!("playing....");
            let _ = play_shutdown_rx
                .blocking_recv()
                .or_else(|error| -> Result<(), _> {
                    tracing::error!(?error, "failed to receive play shutdown");
                    Ok::<(), RecvError>(())
                });

            tracing::info!("done playing....");
        });

        Ok(())
    }

    pub fn to_list_item(&self, index: usize) -> ListItem {
        let bg_color = match index % 2 {
            0 => NORMAL_ROW_COLOR,
            _ => ALT_ROW_COLOR,
        };
        let line = Line::styled(format!(" * {} - {}", self.name, self.url), TEXT_COLOR);

        let list_item = ListItem::new(line);
        list_item.bg(bg_color)
    }
}

impl From<ApiStation> for RadioStation {
    fn from(value: ApiStation) -> Self {
        Self {
            name: value.name,
            stationuuid: value.stationuuid,
            url: value.url,
            codec: value.codec,
            bitrate: value.bitrate,
            homepage: value.homepage,
            tags: value.tags,
            countrycode: value.countrycode,
            languagecodes: value.languagecodes,
            votes: value.votes,
        }
    }
}
