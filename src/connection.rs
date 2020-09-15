use super::config::Credential;
use super::protocols::{self, auth::StartupMessage, stream};
use async_std::io::Error as AsyncError;
use async_std::net::{TcpStream, ToSocketAddrs};
use async_std::prelude::*;

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

    pub async fn startup(
        &mut self,
        cred: Option<&Credential>,
        db: Option<&str>,
    ) -> Result<(), ConnectionError> {
        match cred {
            None => Ok(()),
            Some(Credential::UserPass(user, pass)) => {
                let m = StartupMessage::new(user, db);
                let m = protocols::to_message_with_len(&m, 0).unwrap();
                self.stream
                    .write_all(&m)
                    .await
                    .map_err(|e| ConnectionError::WriteError(e))?;

                let resp = stream::read_from_backend(&mut self.stream)
                    .await
                    .map_err(|e| ConnectionError::ReadError(e))?;

                println!("Response: len: {}, {:?}", resp.len(), resp);
                Ok(())
            }
        }
    }
}

#[derive(Debug)]
pub enum ConnectionError {
    TcpConnect(AsyncError),
    WriteError(std::io::Error),
    ReadError(std::io::Error),
    Unknown,
}
