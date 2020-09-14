use super::pool::*;
use super::config::*;

pub struct Client{
    pub pool: Pool,
}

impl Client {
    pub fn new<C: ToPqConfig>(conf: C, pool_size: usize) -> Result<Client, ConfParseError> {
        Ok(Client{
            pool: Pool::new(conf, pool_size)?,
        })
    }

    pub async fn get_conn(&self) -> Result<PooledConnection, ConnectionPoolError>  {
        self.pool.get_conn().await
    }
}

#[derive(Debug)]
pub enum ClientError {
}
