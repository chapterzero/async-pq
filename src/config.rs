use async_std::net::{SocketAddr, ToSocketAddrs};
use async_std::task;
use regex::Regex;

pub trait ToPqConfig {
    fn to_pq_config(&self) -> Result<PqConfig, ConfParseError>;
}

#[derive(Debug)]
pub struct PqConfig {
    pub address: SocketAddr,
    pub cred: Option<Credential>,
    pub dbname: Option<String>,
}

#[derive(Debug)]
pub enum Credential {
    UserPass(String, Option<String>),
}

#[derive(Debug)]
pub enum ConfParseError {
    // when parsing hostname / ip address port pair into socket address
    ResolveError(std::io::Error),
    // parsing successful but no host from iterator
    NoHost,
}

use std::fmt;

impl fmt::Display for ConfParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Error when parsing configuration")
    }
}

impl std::error::Error for ConfParseError {}

impl ToPqConfig for &str {
    // valid str
    // postgresql://
    // postgresql:///mydb
    // postgresql://localhost
    // postgresql://localhost:5433
    // postgresql://localhost/mydb
    // postgresql://user@localhost
    // postgresql://user:secret@localhost
    // postgresql://other@localhost/otherdb
    fn to_pq_config(&self) -> Result<PqConfig, ConfParseError> {
        lazy_static! {
            static ref RE: Regex =
                Regex::new(r#"^postgresql://([^@]+@)?([^/]+)?(/[\w]+)?$"#).unwrap();
        }
        let captures = RE.captures(self).unwrap();

        // resolve host and port into socket address
        let mut host_port = captures
            .get(2)
            .map(|c| c.as_str())
            .unwrap_or("127.0.0.1:5432")
            .split(":");
        let host = host_port.next().unwrap();
        let port = host_port.next().unwrap_or("5432");
        let socket: SocketAddr = task::block_on(async {
            let mut h = format!("{}:{}", host, port)
                .to_socket_addrs()
                .await
                .map_err(|e| ConfParseError::ResolveError(e))?;
            h.next().ok_or(ConfParseError::NoHost)
        })?;

        let cred = captures.get(1).map(|c| {
            let c = c.as_str();
            let mut split = c[..c.len() - 1].split(":");
            Credential::UserPass( 
                split.next().unwrap().to_string(),
                split.next().map(String::from),
            )
        });
        let dbname = captures.get(3).map(|c| String::from(&c.as_str()[1..]));

        Ok(PqConfig {
            address: socket,
            cred,
            dbname,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_std::net::{Ipv4Addr, Ipv6Addr};

    #[test]
    fn test_parse_default() {
        let conf = "postgresql://".to_pq_config().unwrap();
        assert_eq!("127.0.0.1".parse::<Ipv4Addr>().unwrap(), conf.address.ip());
        assert_eq!(5432, conf.address.port());
        assert!(conf.cred.is_none());
        assert!(conf.dbname.is_none());
    }

    #[test]
    fn test_parse_no_host() {
        let conf = "postgresql:///mydb".to_pq_config().unwrap();
        assert_eq!("127.0.0.1".parse::<Ipv4Addr>().unwrap(), conf.address.ip());
        assert_eq!(5432, conf.address.port());
        assert!(conf.cred.is_none());
        assert_eq!("mydb", conf.dbname.unwrap());
    }

    #[test]
    fn test_parse_single_host() {
        let conf = "postgresql://localhost".to_pq_config().unwrap();
        assert_eq!("::1".parse::<Ipv6Addr>().unwrap(), conf.address.ip());
        assert_eq!(5432, conf.address.port());
        assert!(conf.cred.is_none());
        assert!(conf.dbname.is_none())
    }

    #[test]
    fn test_parse_host_with_port() {
        let conf = "postgresql://localhost:1123".to_pq_config().unwrap();
        assert_eq!("::1".parse::<Ipv6Addr>().unwrap(), conf.address.ip());
        assert_eq!(1123, conf.address.port());
        assert!(conf.cred.is_none());
        assert!(conf.dbname.is_none())
    }

    #[test]
    fn test_parse_host_db() {
        let conf = "postgresql://localhost/mydb".to_pq_config().unwrap();
        assert_eq!("::1".parse::<Ipv6Addr>().unwrap(), conf.address.ip());
        assert_eq!(5432, conf.address.port());
        assert!(conf.cred.is_none());
        assert_eq!("mydb", conf.dbname.unwrap());
    }

    #[test]
    fn test_parse_host_user() {
        let conf = "postgresql://user@localhost".to_pq_config().unwrap();
        assert_eq!("::1".parse::<Ipv6Addr>().unwrap(), conf.address.ip());
        assert_eq!(5432, conf.address.port());
        match conf.cred {
            None => panic!("Should not be none"),
            Some(Credential::UserPass(user, pass)) => {
                assert_eq!("user", user);
                assert!(pass.is_none());
            }
        }
        assert!(conf.dbname.is_none())
    }

    #[test]
    fn test_parse_host_user_pass() {
        let conf = "postgresql://user:secret@localhost".to_pq_config().unwrap();
        assert_eq!("::1".parse::<Ipv6Addr>().unwrap(), conf.address.ip());
        assert_eq!(5432, conf.address.port());
        match conf.cred {
            None => panic!("Should not be none"),
            Some(Credential::UserPass(user, pass)) => {
                assert_eq!("user", user);
                assert_eq!("secret", pass.unwrap());
            }
        }
        assert!(conf.dbname.is_none())
    }

    #[test]
    fn test_parse_complete() {
        let conf = "postgresql://user2:s3cret@33.3.1.1:3223/mydb"
            .to_pq_config()
            .unwrap();
        assert_eq!("33.3.1.1".parse::<Ipv4Addr>().unwrap(), conf.address.ip());
        assert_eq!(3223, conf.address.port());
        match conf.cred {
            None => panic!("Should not be none"),
            Some(Credential::UserPass(user, pass)) => {
                assert_eq!("user2", user);
                assert_eq!("s3cret", pass.unwrap());
            }
        }
        assert_eq!("mydb", conf.dbname.unwrap());
    }
}
