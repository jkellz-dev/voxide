use std::{io::Write, sync::Arc, thread, time::Duration};

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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RadioStation {
    pub name: String,
    pub url: String,
}

impl RadioStation {
    pub fn new<T: ToString>(url: T, name: T) -> Self {
        Self {
            name: name.to_string(),
            url: url.to_string(),
        }
    }
    pub async fn play(
        &mut self,
        mut download_shutdown_rx: broadcast::Receiver<()>,
        mut play_shutdown_rx: broadcast::Receiver<()>,
    ) -> Result<(), Error> {
        tracing::info!(station = ?self, "playing");
        let client = reqwest::Client::new();
        let mut response = client
            .get(&self.url)
            .header(header::CONNECTION, "keep-alive")
            .send()
            .await?;

        tracing::info!(?response, "got response");

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
                                        Ok(n) => tracing::trace!("pushed chunk: {}", n),
                                        Err(e) => tracing::error!("error {:?}", e),
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

        tracing::info!("waiting for chunks");

        while audio_stream.len()? < 1024 * 10 {
            tokio::task::yield_now().await;
        }

        let blocking_task = tokio::task::spawn_blocking(move || {
            // This is running on a thread where blocking is fine.

            let (stream, stream_handle) = OutputStream::try_default().unwrap();
            let sink = Sink::try_new(&stream_handle).unwrap();

            tracing::info!("setting up decoder");
            let decoder = Decoder::new_mp3(audio_stream).unwrap();
            sink.append(decoder);

            play_shutdown_rx.blocking_recv();
            // sink.sleep_until_end();
            tracing::info!("done playing");
        });

        tracing::info!("playing");

        // let _ = shutdown_rx.blocking_recv();
        //
        // sink.stop();
        //
        // let _ = shutdown_send.send(());

        // tokio::select! {
        //     _ = shutdown_rx.recv() => {
        //         tracing::info!("shutting down");
        //         sink.stop();
        //     }
        // }

        // sink.sleep_until_end();

        // tracing::info!("done playing");
        Ok(())
    }

    pub fn to_list_item(&self, index: usize) -> ListItem {
        let bg_color = match index % 2 {
            0 => NORMAL_ROW_COLOR,
            _ => ALT_ROW_COLOR,
        };
        let line = Line::styled(format!(" * {} - {}", self.name, self.url), TEXT_COLOR);
        // let line = match self.status {
        //     Status::Todo => Line::styled(format!(" ☐ {}", self.todo), TEXT_COLOR),
        //     Status::Completed => Line::styled(
        //         format!(" ✓ {}", self.todo),
        //         (COMPLETED_TEXT_COLOR, bg_color),
        //     ),
        // };

        let list_item = ListItem::new(line);
        list_item.bg(bg_color)
    }
}

impl From<ApiStation> for RadioStation {
    fn from(value: ApiStation) -> Self {
        Self {
            name: value.name,
            url: value.url,
        }
    }
}
