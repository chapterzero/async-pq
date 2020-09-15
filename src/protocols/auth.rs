use serde::Serialize;

pub enum AuthResponse {
    ErrorResponse,
    AuthenticationOk,
    AuthenticationCleartextPassword,
    AuthenticationMD5Password,
    NotImplemented,
}

// intermediate start up phase between
// authentication ok and ready for query
pub enum IntermediateResponse {
    BackendKeyData,
    ParameterStatus,
    ReadyForQuery,
    ErrorResponse,
    NoticeResponse,
    NotImplemented,
}

const STARTUP_VERSION: u32 = 196608;
static STARTUP_USER: &'static [u8] = b"user\0";
static STARTUP_DBNAME: &'static [u8] = b"database\0";

#[derive(Serialize)]
pub struct StartupMessage<'a> {
    ver: u32,
    param: Vec<(&'static [u8], &'a str)>,
}

impl<'a> StartupMessage<'a> {
    pub fn new(user: &'a str, dbname: Option<&'a str>) -> StartupMessage<'a> {
        let mut param = vec![(STARTUP_USER, user)];
        if let Some(dbname) = dbname {
            param.push((STARTUP_DBNAME, dbname));
        }

        StartupMessage {
            ver: STARTUP_VERSION,
            param,
        }
    }
}
