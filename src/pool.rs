use super::connection::{Connection, ConnectionError};
use super::config::*;
use std::sync::{Arc, Mutex};

#[derive(Clone)]
pub struct Pool {
    inner: Arc<Inner>,
}

struct Inner {
    conf: PqConfig,
    // vector of free connection in pool
    conns: Mutex<Vec<Connection>>,
    // 0: total of connection allocated
    // 1: total of connection pending
    conn_allocated: Mutex<(usize, usize)>,
    max_conn: usize,
}

impl Pool {
    pub fn new<C: ToPqConfig>(conf: C, max_conn: usize) -> Result<Pool, ConfParseError> {
        Ok(Pool {
            inner: Arc::new(Inner {
                conf: conf.to_pq_config()?,
                conns: Mutex::new(Vec::with_capacity(max_conn)),
                conn_allocated: Mutex::new((0, 0)),
                max_conn,
            }),
        })
    }

    pub async fn get_conn(&self) -> Result<PooledConnection, ConnectionPoolError> {
        let conn: Option<Connection> = {
            let mut conns = self.inner.conns.lock().unwrap();
            conns.pop()
        };

        match conn {
            Some(c) => Ok(PooledConnection {
                pool: self.clone(),
                conn: Some(c),
            }),
            None => {
                let mut allocated = self.inner.conn_allocated.lock().unwrap();

                // max conn check
                if allocated.0 + allocated.1 >= self.inner.max_conn {
                    return Err(ConnectionPoolError::Exhausted);
                }

                // allocate new connection
                // add new pending, drop the mutex
                allocated.1 += 1;
                debug!(
                    "Creating new connection, total allocated: {}, pending: {}",
                    allocated.0, allocated.1
                );
                drop(allocated);

                let mut conn = Connection::new(self.inner.conf.address)
                    .await
                    .map_err(|e| ConnectionPoolError::Connection(e))?;
                conn.startup(self.inner.conf.cred.as_ref(), self.inner.conf.dbname.as_deref())
                    .await
                    .map_err(|e| ConnectionPoolError::Connection(e))?;
;

                let mut allocated = self.inner.conn_allocated.lock().unwrap();
                allocated.0 += 1;
                allocated.1 -= 1;
                debug!(
                    "Allocated new connection, total allocated: {}, pending: {}",
                    allocated.0, allocated.1
                );

                Ok(PooledConnection {
                    pool: self.clone(),
                    conn: Some(conn),
                })
            }
        }
    }

    pub fn put_back(&self, conn: Connection) {
        let mut vec = self.inner.conns.lock().unwrap();
        debug!("#REMOVE total in pools: {}", vec.len());
        vec.push(conn);
    }
}

pub struct PooledConnection {
    pool: Pool,
    conn: Option<Connection>,
}

impl Drop for PooledConnection {
    fn drop(&mut self) {
        self.pool.put_back(self.conn.take().unwrap());
    }
}

#[derive(Debug)]
pub enum ConnectionPoolError {
    Connection(ConnectionError),
    Exhausted,
}

use std::fmt;

impl fmt::Display for ConnectionPoolError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Connection pool error")
    }
}

impl std::error::Error for ConnectionPoolError {}
