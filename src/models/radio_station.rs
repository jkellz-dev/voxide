use std::{io::Write, thread, time::Duration};

use log::debug;
use radiobrowser::ApiStation;
use reqwest::header;
use rodio::{Decoder, OutputStream, Sink};

use crate::errors::Error;

use super::audio_stream::AudioStream;

pub struct RadioStation {
    pub name: String,
    pub url: String,
    sink: Option<Sink>,
}

impl RadioStation {
    pub fn new<T: ToString>(url: T, name: T) -> Self {
        Self {
            // buffer,
            name: name.to_string(),
            url: url.to_string(),
            sink: None,
        }
    }
    pub async fn play(&mut self) -> Result<OutputStream, Error> {
        let client = reqwest::Client::new();
        let mut response = client
            .get(&self.url)
            .header(header::CONNECTION, "keep-alive")
            .send()
            .await?;

        if response.status() != 200 {
            return Err(Error::Http(response.status()));
        }

        let (_stream, stream_handle) = OutputStream::try_default().unwrap();
        let sink = Sink::try_new(&stream_handle).unwrap();

        let audio_stream = AudioStream::new();

        let buf = audio_stream.get_buf();

        tokio::spawn(async move {
            while let Some(chunk) = response.chunk().await.unwrap() {
                debug!("got chunk: {}", chunk.len());
                let mut guard = buf.lock().expect("failed to lock buffer");
                let result = guard.write(chunk.as_ref());
                match result {
                    Ok(n) => debug!("pushed chunk: {}", n),
                    Err(e) => debug!("error {:?}", e),
                }
            }
        });

        while audio_stream.len()? < 1024 * 10 {
            thread::sleep(Duration::from_millis(100));
        }

        let decoder = Decoder::new_mp3(audio_stream).unwrap();
        sink.append(decoder);

        self.sink = Some(sink);

        Ok(_stream)
    }
}

impl From<ApiStation> for RadioStation {
    fn from(value: ApiStation) -> Self {
        Self {
            name: value.name,
            url: value.url,
            sink: None,
        }
    }
}
