#[macro_use]
extern crate rocket;

use binance::account::Account;
use binance::api::Binance;
use binance::market::Market;
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

#[get("/mytrades/binance/<ticker>")]
async fn binance_trades(ticker: String) -> String {
    let result = std::thread::spawn(|| {
        let api_key = Some(std::env::var("BINANCE_API_KEY").unwrap());
        let secret_key = Some(std::env::var("BINANCE_SECRET_KEY").unwrap());
        let account: Account = Binance::new(api_key, secret_key);
        let history = account.trade_history(ticker).unwrap();
        history
    })
    .join()
    .unwrap();

    serde_json::to_string(&result).unwrap()
}

#[get("/mytrades/binance")]
async fn binance_trades_all() -> String {
    #[derive(serde::Serialize)]
    struct Trade {
        symbol: String,
        price: f64,
        qty: f64,
        date: String,
        timestamp: u64,
        transaction_cost: String,
        transaction_asset: String,
    }
    let symbols = vec![
        "BNBUSDT", "ETHUSDT", "DOGEUSDT", "TRXUSDT", "UNIUSDT", "BTCUSDT",
    ];
    let result = std::thread::spawn(|| {
        let api_key = Some(std::env::var("BINANCE_API_KEY").unwrap());
        let secret_key = Some(std::env::var("BINANCE_SECRET_KEY").unwrap());
        let account: Account = Binance::new(api_key, secret_key);
        let mut trades = symbols
            .into_iter()
            .map(|symbol| {
                account
                    .trade_history(symbol)
                    .unwrap()
                    .into_iter()
                    .map(|t| Trade {
                        symbol: format!("F:{}", symbol),
                        price: t.price,
                        qty: t.qty,
                        date: chrono::NaiveDateTime::from_timestamp((t.time / 1000) as i64, 0)
                            .format("%m/%d/%Y")
                            .to_string(),
                        timestamp: t.time,
                        transaction_cost: t.commission,
                        transaction_asset: t.commission_asset,
                    })
                    .collect::<Vec<Trade>>()
            })
            .flatten()
            .collect::<Vec<_>>();
        trades.sort_by_key(|t| t.timestamp);
        trades
    })
    .join()
    .unwrap();

    serde_json::to_string(&result).unwrap()
}

#[get("/price/<symbol>")]
async fn get_ticker_price(symbol: String) -> String {
    if let Some((exchange, ticker)) = symbol.split_once(':') {
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
            "B" => {
                let t1 = ticker.to_string();
                let price = std::thread::spawn(move || {
                    let market: Market = Binance::new(None, None);
                    let price = market.get_price(t1).unwrap();
                    price.price
                })
                .join()
                .unwrap();
                price.to_string()
            }
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
    dotenv::dotenv().ok();
    rocket::build().mount(
        "/",
        routes![index, get_ticker_price, binance_trades, binance_trades_all],
    )
}
