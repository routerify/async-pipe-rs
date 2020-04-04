use state::State;
use std::sync::{Arc, Mutex};

pub use self::reader::PipeReader;
pub use self::writer::PipeWriter;

mod reader;
mod state;
mod writer;

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
        state: Arc::clone(&shared_state),
    };

    let r = PipeReader {
        state: Arc::clone(&shared_state),
    };

    (w, r)
}
