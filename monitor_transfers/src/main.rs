use alloy::{
    contract::{ContractInstance, Interface},
    dyn_abi::DynSolValue,
    primitives::{Address, U256},
    providers::{Provider, ProviderBuilder, WsConnect},
    rpc::types::{BlockNumberOrTag, Filter},
    network::EthereumWallet,
    signers::local::PrivateKeySigner,
};
use eyre::Result;
use futures_util::StreamExt;
use std::str::FromStr;
use dotenv::dotenv;
use std::env;

#[tokio::main]
async fn main() -> Result<()> {
    dotenv().ok(); // Load environment variables from .env file

    // Read provider URL from environment variables
    let provider_url = env::var("PROVIDER_URL")?;

    // Initialize WebSocket provider
    let ws = WsConnect::new(&provider_url);
    let provider = ProviderBuilder::new()
        .with_recommended_fillers()
        .on_ws(ws).await?;

    // WETH contract address
    let weth_address = Address::from_str("4200000000000000000000000000000000000006")?;

    // ABI for the token contract
    let token_abi = r#"[{"anonymous":false,"inputs":[{"indexed":true,"internalType":"address","name":"from","type":"address"},{"indexed":true,"internalType":"address","name":"to","type":"address"},{"indexed":false,"internalType":"uint256","name":"value","type":"uint256"}],"name":"Transfer","type":"event"}]"#;
    let token_interface = Interface::new(serde_json::from_str(token_abi)?);
    let weth_contract = ContractInstance::new(weth_address, provider.clone(), token_interface);

    // Create a filter to watch for Transfer events
    let filter = Filter::new()
        .address(weth_address)
        .event("Transfer(address,address,uint256)")
        .from_block(BlockNumberOrTag::Latest);

    // Subscribe to logs
    let sub = provider.subscribe_logs(&filter).await?;
    let mut stream = sub.into_stream();

    println!("Monitoring Transfer events for WETH...");

    while let Some(log) = stream.next().await {
        println!("log");
        // Extract and print Transfer event details
        // if let Some((from, to, value)) = log.inner.data.decode::<(Address, Address, U256)>() {
        //     let amount = value.to_string(); // Convert U256 to string for display
        //     println!("Transfer event detected. From: {:?}, To: {:?}, Amount: {}", from, to, amount);
        // } else {
        //     println!("Failed to decode Transfer event");
        // }
    }

    Ok(())
} 