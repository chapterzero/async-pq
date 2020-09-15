use async_std::net::TcpStream;
use async_std::prelude::*;

const DEF_BUF_LEN: usize = 32;

pub async fn read_from_backend(stream: &mut TcpStream) -> Result<Vec<u8>, std::io::Error> {
    read_from_backend_buf(stream, DEF_BUF_LEN).await
}

pub async fn read_from_backend_buf(
    stream: &mut TcpStream,
    buf_size: usize,
) -> Result<Vec<u8>, std::io::Error> {
    // // first byte indicate the response type (either R / E)
    // // 2nd u32 indicate message length (not including the first byte)
    let mut v = Vec::with_capacity(buf_size);
    let mut known_len: Option<usize> = None;
    loop {
        let mut buf = vec![0u8; buf_size];
        let n = stream.read(&mut buf).await?;

        if known_len.is_none() {
            let mut msg_len = [0u8; 4];
            msg_len.copy_from_slice(&buf[1..5]);
            known_len = Some(u32::from_be_bytes(msg_len) as usize)
        }

        v.extend(&buf[..n]);

        if n < buf_size || v.len() - 1 == (known_len.unwrap()) {
            break
        }
    }
    Ok(v)
}
