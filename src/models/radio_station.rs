use std::io::Write;
use std::{thread, time::Duration};

use reqwest::header;
use rodio::{Decoder, OutputStream, Sink};

use crate::errors::Error;

use super::audio_stream::AudioStream;

pub struct RadioStation<'a> {
    url: &'a str,
    sink: Option<Sink>,
}

impl<'a> RadioStation<'a> {
    pub fn new(url: &'a str) -> Self {
        Self {
            // buffer,
            url,
            sink: None,
        }
    }
    pub async fn play(&mut self) -> Result<OutputStream, Error> {
        let client = reqwest::Client::new();
        let mut response = client
            .get(self.url)
            .header(header::CONNECTION, "keep-alive")
            .send()
            .await?;

        if response.status() != 200 {
            return Err(Error::HttpError(response.status()));
        }

        let (_stream, stream_handle) = OutputStream::try_default().unwrap();
        let sink = Sink::try_new(&stream_handle).unwrap();

        let audio_stream = AudioStream::new();

        let buf = audio_stream.get_buf();

        tokio::spawn(async move {
            while let Some(chunk) = response.chunk().await.unwrap() {
                println!("got chunk: {}", chunk.len());
                let mut guard = buf.lock().expect("failed to lock buffer");
                let result = guard.write(chunk.as_ref());
                match result {
                    Ok(n) => println!("pushed chunk: {}", n),
                    Err(e) => println!("error {:?}", e),
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
