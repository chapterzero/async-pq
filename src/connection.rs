use async_std::io::Error as AsyncError;
use async_std::net::{TcpStream, ToSocketAddrs};

pub struct Connection {
    stream: TcpStream,
}

impl Connection {
    pub async fn new<A: ToSocketAddrs>(address: A) -> Result<Connection, ConnectionError> {
        Ok(Connection {
            stream: TcpStream::connect(address)
                .await
                .map_err(|e| ConnectionError::TcpConnect(e))?,
        })
    }

    pub async fn open(&mut self) -> Result<(), ConnectionError> {
        Ok(())
    }
}

#[derive(Debug)]
pub enum ConnectionError {
    TcpConnect(AsyncError),
    Unknown,
}
