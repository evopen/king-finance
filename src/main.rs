#[macro_use]
extern crate rocket;

use rocket::routes;

async fn get_string(req: String) -> String {
    reqwest::get(req).await.unwrap().text().await.unwrap()
}

async fn get_sina_current_price(raw: String) -> Option<String> {
    dbg!(&raw);
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

#[get("/price/<ticker>")]
async fn get_ticker_price(ticker: String) -> String {
    if let Some((exchange, ticker)) = ticker.split_once(':') {
        match exchange {
            "SHSE" => get_sina_current_price(
                get_string(format!("http://hq.sinajs.cn/list=sh{}", ticker)).await,
            )
            .await
            .unwrap_or("error".into()),
            "SZSE" => get_sina_current_price(
                get_string(format!("http://hq.sinajs.cn/list=sz{}", ticker)).await,
            )
            .await
            .unwrap_or("error".into()),
            "F" => get_sina_current_price(
                get_string(format!("http://hq.sinajs.cn/list=of{}", ticker)).await,
            )
            .await
            .unwrap_or("error".into()),
            _ => "unknown stock exchange".into(),
        }
    } else {
        "must have exchange symbol".into()
    }
}

#[get("/")]
async fn index() -> &'static str {
    "Hello, world!"
}

#[launch]
fn rocket() -> _ {
    rocket::build().mount("/", routes![index, get_ticker_price])
}
