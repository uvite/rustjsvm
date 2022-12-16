// use tide;
// #[async_std::main]
// async fn main() -> Result<(), std::io::Error> {
//     //tide_rhai::log::start();
//
//     let mut app = tide::new();
//     app.at("/*")
//         .get(RhaiDir::new("/*", "./examples/app/").unwrap());
//     // log::info!("Visit samples:");
//     // log::info!("http://127.0.0.1:8080/helloworld.rhai:");
//     // log::info!("http://127.0.0.1:8080/headers.rhai");
//     // log::info!("http://127.0.0.1:8080/fetch.rhai");
//     app.listen("127.0.0.1:8080").await?;
//    // Ok(())
// }


use tide_rhai::RhaiDir;

use tide::Request;
use tide::prelude::*;

#[derive(Debug, Deserialize)]
struct Animal {
    name: String,
    legs: u16,
}

#[async_std::main]
async fn main() -> tide::Result<()> {
    let mut app = tide::new();
    app.at("/orders/shoes").post(order_shoes);
    app.at("/*")
    .get(RhaiDir::new("/*", "./app/").unwrap());
    app.listen("127.0.0.1:8080").await?;
    Ok(())
}

async fn order_shoes(mut req: Request<()>) -> tide::Result {
    let Animal { name, legs } = req.body_json().await?;
    Ok(format!("Hello, {}! I've put in an order for {} shoes", name, legs).into())
}
