#[macro_use] extern crate log;
use async_pq::Client;
use async_std::task;
use futures::future::join_all;
use std::{thread, time};

fn main() {
    env_logger::init();
    let st = time::Instant::now();
    let conc = 10;
    let client = Client::new("postgresql://myuser:secret@localhost:15432/mydb", conc).unwrap();
    let mut futures = vec![];
    for i in 0..conc {
        let pool = client.pool.clone();
        futures.push(query_something(pool, i))
    }

    task::block_on(async {
        let res = join_all(futures).await;
        info!("{:?}", res);
    });

    info!("All finished");
    info!("Elapsed: {:?}", st.elapsed());
    thread::sleep(time::Duration::from_secs(3600))
}

async fn query_something(p: async_pq::Pool, i: usize) -> Result<(), Box<dyn std::error::Error>> {
    debug!("Futures: {} getting connection", i);
    let mut conn = p.get_conn().await?;
    Ok(())
}
