use futures::future::join_all;
use reqwest::Client;
use serde::Deserialize;
use std::error::Error;
use std::fs;

#[derive(Debug, Deserialize)]
struct Config {
    wallets: Vec<String>,
    rpc_url: String,
}

#[derive(Debug, Deserialize)]
struct RpcResponse {
    result: u64,
}

async fn get_balance(client: &Client, rpc_url: &str, pubkey: &str) -> Result<u64, Box<dyn Error>> {
    let payload = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "getBalance",
        "params": [pubkey]
    });

    let response = client
        .post(rpc_url)
        .json(&payload)
        .send()
        .await?
        .json::<serde_json::Value>()
        .await?;

    if let Some(balance) = response
        .get("result")
        .and_then(|r| r.get("value"))
        .and_then(|v| v.as_u64())
    {
        Ok(balance)
    } else {
        Err("Invalid response from RPC".into())
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Загружаем конфиг
    let config_str = fs::read_to_string("config.yaml")?;
    let config: Config = serde_yaml::from_str(&config_str)?;

    let client = Client::new();

    let tasks = config.wallets.iter().map(|wallet| {
        let client = &client;
        let rpc_url = config.rpc_url.clone();
        let wallet = wallet.clone();
        async move {
            match get_balance(client, &rpc_url, &wallet).await {
                Ok(balance) => println!("{}: {} lamports", wallet, balance),
                Err(e) => eprintln!("Failed to get balance for {}: {}", wallet, e),
            }
        }
    });

    join_all(tasks).await;

    Ok(())
}
