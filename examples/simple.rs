use async_pq::Client;
use async_std::task;
use async_std::net::TcpStream;
use async_std::prelude::*;
use std::str;

fn main() {
    let mut client = Client::new("postgresql://root:secret@localhost:15432/mydb", 10).unwrap();
    task::block_on(async {
        let mut conn = client.get_conn().await.unwrap();
        conn.open().await.unwrap();
    });
}
