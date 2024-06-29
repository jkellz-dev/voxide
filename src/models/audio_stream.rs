use std::{
    collections::VecDeque,
    io::{Read, Seek},
    sync::Arc,
};

use tracing::debug;

use crate::errors::Error;

pub struct AudioStream {
    buf: Arc<std::sync::Mutex<VecDeque<u8>>>,
}

impl AudioStream {
    pub fn new() -> Self {
        let buf = Arc::new(std::sync::Mutex::new(VecDeque::<u8>::new()));

        Self { buf }
    }

    pub fn get_buf(&self) -> Arc<std::sync::Mutex<VecDeque<u8>>> {
        self.buf.clone()
    }

    pub fn len(&self) -> Result<usize, Error> {
        Ok(self
            .buf
            .lock()
            .map_err(|e| Error::Lock(e.to_string()))?
            .len())
    }
}

impl Seek for AudioStream {
    fn seek(&mut self, _pos: std::io::SeekFrom) -> std::io::Result<u64> {
        // Err(std::io::Error::new(
        //     std::io::ErrorKind::Other,
        //     "Seek not supported",
        // ))
        Ok(0)
    }
}

impl Read for AudioStream {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let mut guard = self.buf.lock().expect("failed to lock buffer");
        debug!("reading: {}", buf.len());
        guard.read(buf)
    }
}
