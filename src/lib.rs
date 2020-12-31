//! Creates an asynchronous piped reader and writer pair using `tokio.rs`.
//!
//! # Examples
//!
//! ```
//! # async fn run() {
//! use async_pipe;
//! use tokio::io::{AsyncWriteExt, AsyncReadExt};
//!
//! let (mut w, mut r) = async_pipe::pipe();
//!  
//! tokio::spawn(async move {
//!     w.write_all(b"hello world").await.unwrap();
//! });
//!  
//! let mut v = Vec::new();
//! r.read_to_end(&mut v).await.unwrap();
//!
//! println!("Received: {:?}", String::from_utf8(v));
//! # }
//!
//! tokio::runtime::Runtime::new().unwrap().block_on(run());
//! ```

use state::State;
use std::sync::{Arc, Mutex};

pub use self::reader::PipeReader;
pub use self::writer::PipeWriter;

mod reader;
mod state;
mod writer;

/// Creates a piped pair of an [`AsyncWrite`](https://docs.rs/tokio/0.2.16/tokio/io/trait.AsyncWrite.html) and an [`AsyncRead`](https://docs.rs/tokio/0.2.15/tokio/io/trait.AsyncRead.html).
pub fn pipe() -> (PipeWriter, PipeReader) {
    let shared_state = Arc::new(Mutex::new(State {
        reader_waker: None,
        writer_waker: None,
        data: None,
        done_reading: false,
        read: 0,
        done_cycle: true,
        closed: false,
    }));

    let w = PipeWriter {
        state: shared_state.clone()
    };

    let r = PipeReader {
        state: shared_state.clone(),
    };

    (w, r)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::io::{AsyncWriteExt, AsyncReadExt};

    #[tokio::test]
    async fn should_read_expected_text() {
        const EXPECTED: &'static str = "hello world";

        let (mut w, mut r) = pipe();

        tokio::spawn(async move {
            w.write_all(EXPECTED.as_bytes()).await.unwrap();
        });

        let mut v = Vec::new();
        r.read_to_end(&mut v).await.unwrap();
        let actual = String::from_utf8(v).unwrap();
        assert_eq!(EXPECTED, actual.as_str());
    }
}
