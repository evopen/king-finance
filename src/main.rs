use std::convert::Infallible;

use serde::{Deserialize, Serialize};
use warp::Filter;

#[derive(Deserialize, Serialize)]
struct Finance {
    ticker: String,
}

#[tokio::main]
async fn main() {
    run().await;
}

async fn get_string(req: String) -> String {
    reqwest::get(req).await.unwrap().text().await.unwrap()
}

async fn get_sina_current_p(raw: String) -> Option<String> {
    let arr = raw
        .split_once('=')
        .unwrap()
        .1
        .split(',')
        .collect::<Vec<_>>();
    if arr.len() < 5 {
        None
    } else {
        Some(arr[3].into())
    }
}

async fn get_ticker_p(f: Finance) -> Result<impl warp::Reply, Infallible> {
    if let Some(result) =
        get_sina_current_p(get_string(format!("http://hq.sinajs.cn/list=sh{}", f.ticker)).await)
            .await
    {
        Ok(result)
    } else if let Some(result) =
        get_sina_current_p(get_string(format!("http://hq.sinajs.cn/list=sz{}", f.ticker)).await)
            .await
    {
        Ok(result)
    } else if let Some(result) =
        get_sina_current_p(get_string(format!("http://hq.sinajs.cn/list=of{}", f.ticker)).await)
            .await
    {
        Ok(result)
    } else {
        Ok("error".to_string())
    }
}

async fn run() {
    // GET /hello/warp => 200 OK with body "Hello, warp!"
    let hello = warp::get()
        .and(warp::path("finance"))
        .and(warp::query::<Finance>())
        .and_then(get_ticker_p);

    warp::serve(hello).run(([0, 0, 0, 0], 3000)).await;
}
