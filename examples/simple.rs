use async_pq::Client;
use async_std::task;

fn main() {
    let mut client = Client::new();
    let mut conn = client.get_conn();
    task::block_on(async{
        conn.open().await.unwrap();
    });
}
