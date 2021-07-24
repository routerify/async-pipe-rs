use async_pipe;
use tokio::io::{AsyncWriteExt, AsyncReadExt};

#[tokio::main]
async fn main() {
    let (mut w, mut r) = async_pipe::pipe();

    tokio::spawn(async move {
        w.write_all(b"hello world").await.unwrap();
    });

    let mut v = Vec::new();
    r.read_to_end(&mut v).await.unwrap();
    println!("Received: {:?}", String::from_utf8(v));
}
