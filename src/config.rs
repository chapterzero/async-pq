use async_std::net::IpAddr;

pub trait ToPqConfig {
    fn to_pq_config(&self) -> Result<PqConfig, ConfParseError>;
}

#[derive(Debug)]
pub struct PqConfig {
    pub address: (IpAddr, u16),
    pub cred: Option<Credential>,
    pub dbname: String,
}

#[derive(Debug)]
pub struct Credential {
    pub user: String,
    pub pass: Option<String>,
}

#[derive(Debug)]
pub struct ConfParseError {}

use std::fmt;

impl fmt::Display for ConfParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Error when parsing configuration")
    }
}

impl std::error::Error for ConfParseError{}

impl ToPqConfig for &str {
    fn to_pq_config(&self) -> Result<PqConfig, ConfParseError> {
        Err(ConfParseError{})
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_no_cred() {
        assert_eq!(true, false);
    }
}
