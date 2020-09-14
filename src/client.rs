use super::connection::*;
use super::pool::Pool;
use super::config::*;

pub struct Client{
    conf: PqConfig,
    pool: Pool,
}

impl Client {
    pub fn new<C: ToPqConfig>(conf: C, pool_size: usize) -> Result<Client, ConfParseError> {
        Ok(Client{
            conf: conf.to_pq_config()?,
            pool: Pool::new(pool_size),
        })
    }

    pub async fn get_conn(&mut self) -> Result<Connection, ConnectionError>  {
        let conn = {
            // self.pool.
        };
        Ok(Connection::new(self.conf.address).await?)
    }
}

#[derive(Debug)]
pub enum ClientError {
}
