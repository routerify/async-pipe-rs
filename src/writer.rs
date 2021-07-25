use crate::state::Data;
use crate::state::State;
use std::pin::Pin;
use std::sync::{Arc, Mutex};
use std::task::{Context, Poll};
use tokio::io::{self, AsyncWrite};

/// The write half of the pipe which implements [`AsyncWrite`](https://docs.rs/tokio/0.2.16/tokio/io/trait.AsyncWrite.html).
#[derive(Clone)]
pub struct PipeWriter {
    pub(crate) state: Arc<Mutex<State>>,
}

impl PipeWriter {
    /// Closes the pipe, any further read will return EOF and any further write will raise an error.
    pub fn close(&self) -> io::Result<()> {
        match self.state.lock() {
            Ok(mut state) => {
                state.closed = true;
                self.wake_reader_half(&*state);
                Ok(())
            }
            Err(err) => Err(io::Error::new(
                io::ErrorKind::Other,
                format!(
                    "{}: PipeWriter: Failed to lock the channel state: {}",
                    env!("CARGO_PKG_NAME"),
                    err
                ),
            )),
        }
    }

    /// It returns true if the next data chunk is written and consumed by the reader; Otherwise it returns false.
    pub fn is_flushed(&self) -> io::Result<bool> {
        let state = match self.state.lock() {
            Ok(s) => s,
            Err(err) => {
                return Err(io::Error::new(
                    io::ErrorKind::Other,
                    format!(
                        "{}: PipeWriter: Failed to lock the channel state: {}",
                        env!("CARGO_PKG_NAME"),
                        err
                    ),
                ));
            }
        };

        Ok(state.done_cycle)
    }

    fn wake_reader_half(&self, state: &State) {
        if let Some(ref waker) = state.reader_waker {
            waker.clone().wake();
        }
    }
}

impl Drop for PipeWriter {
    fn drop(&mut self) {
        if let Err(err) = self.close() {
            log::warn!(
                "{}: PipeWriter: Failed to close the channel on drop: {}",
                env!("CARGO_PKG_NAME"),
                err
            );
        }
    }
}

impl AsyncWrite for PipeWriter {
    fn poll_write(self: Pin<&mut Self>, cx: &mut Context, buf: &[u8]) -> Poll<io::Result<usize>> {
        let mut state;
        match self.state.lock() {
            Ok(s) => state = s,
            Err(err) => {
                return Poll::Ready(Err(io::Error::new(
                    io::ErrorKind::Other,
                    format!(
                        "{}: PipeWriter: Failed to lock the channel state: {}",
                        env!("CARGO_PKG_NAME"),
                        err
                    ),
                )))
            }
        }

        if state.closed {
            return Poll::Ready(Err(io::Error::new(
                io::ErrorKind::BrokenPipe,
                format!(
                    "{}: PipeWriter: The channel is closed",
                    env!("CARGO_PKG_NAME")
                ),
            )));
        }

        return if state.done_cycle {
            state.data = Some(Data {
                ptr: buf.as_ptr(),
                len: buf.len(),
            });
            state.done_cycle = false;
            state.writer_waker = Some(cx.waker().clone());

            self.wake_reader_half(&*state);

            Poll::Pending
        } else {
            if state.done_reading {
                let read_bytes_len = state.read;

                state.done_cycle = true;
                state.read = 0;
                state.writer_waker = None;
                state.data = None;
                state.done_reading = false;

                Poll::Ready(Ok(read_bytes_len))
            } else {
                state.writer_waker = Some(cx.waker().clone());
                Poll::Pending
            }
        };
    }

    fn poll_flush(self: Pin<&mut Self>, _cx: &mut Context) -> Poll<io::Result<()>> {
        Poll::Ready(Ok(()))
    }

    fn poll_shutdown(self: Pin<&mut Self>, _cx: &mut Context) -> Poll<io::Result<()>> {
        match self.close() {
            Ok(_) => Poll::Ready(Ok(())),
            Err(err) => Poll::Ready(Err(io::Error::new(
                io::ErrorKind::Other,
                format!(
                    "{}: PipeWriter: Failed to shutdown the channel: {}",
                    env!("CARGO_PKG_NAME"),
                    err
                ),
            ))),
        }
    }
}
