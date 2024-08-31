use reqwest_proxy_pool::ProxyPool;

#[tokio::main]
async fn main() {
    std::env::set_var("RUST_LOG", "INFO,reqwest_proxy_pool=TRACE");
    env_logger::init();

    let pp = ProxyPool::new();

    pp.scraper().await;
}
