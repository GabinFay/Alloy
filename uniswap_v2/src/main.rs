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

    // Read sensitive information from environment variables
    let private_key = env::var("PRIVATE_KEY")?;
    let rpc_url = env::var("RPC_URL")?;
    let wallet_address = env::var("WALLET_ADDRESS")?;

    let signer: PrivateKeySigner = private_key.parse().unwrap();
    let wallet = EthereumWallet::from(signer);
    let ws = WsConnect::new(&rpc_url);
    let provider = ProviderBuilder::new()
        .with_recommended_fillers()
        .wallet(wallet)
        .on_ws(ws).await?;

    // Uniswap V2 Factory Contract
    let factory_address = Address::from_str("8909Dc15e40173Ff4699343b6eB8132c65e18eC6")?;

    // Uniswap V2 Router02 Contract
    let router_address = Address::from_str("4752ba5DBc23f44D87826276BF6Fd6b1C372aD24")?;
    let router_abi = r#"[{"inputs":[{"internalType":"uint256","name":"amountIn","type":"uint256"},{"internalType":"uint256","name":"amountOutMin","type":"uint256"},{"internalType":"address[]","name":"path","type":"address[]"},{"internalType":"address","name":"to","type":"address"},{"internalType":"uint256","name":"deadline","type":"uint256"}],"name":"swapExactTokensForTokensSupportingFeeOnTransferTokens","outputs":[],"stateMutability":"nonpayable","type":"function"}]"#;
    let router_interface = Interface::new(serde_json::from_str(router_abi)?);
    let router_contract = ContractInstance::new(router_address, provider.clone(), router_interface);

    let token0_placeholder = Address::from_str("4200000000000000000000000000000000000006")?; // Token0 fixed

    // Create a filter to watch for PairCreated events
    let filter = Filter::new()
        .address(factory_address)
        .event("PairCreated(address,address,address,uint256)")
        .from_block(BlockNumberOrTag::Latest);

    // Subscribe to logs
    let sub = provider.subscribe_logs(&filter).await?;
    let mut stream = sub.into_stream();

    while let Some(log) = stream.next().await {
        // Print the entire log to explore its structure
        println!("{:?}", log);

        // Extract token1 from the log's topics using the topics() method
        let token1_topic = if let Some(topic) = log.inner.data.topics().get(1) {
            let token1_str = format!("{:?}", topic);
            let token1_address = Address::from_str(&token1_str[26..])?;
            if token1_address != token0_placeholder {
                Some(token1_address)
            } else {
                None
            }
        } else {
            None
        };

        let token1_address = if token1_topic.is_none() {
            if let Some(topic) = log.inner.data.topics().get(2) {
                let token1_str = format!("{:?}", topic);
                let token1_address = Address::from_str(&token1_str[26..])?;
                if token1_address != token0_placeholder {
                    Some(token1_address)
                } else {
                    None
                }
            } else {
                None
            }
        } else {
            token1_topic
        };

        if let Some(token1_address) = token1_address {
            println!("Extracted token1 address: {:?}", token1_address);

            // Perform token-to-token swap
            let amount_in = U256::from_str("10000000000000")?; // 0.00001e18
            let amount_out_min = U256::from(1); // Minimum output amount
            let path: Vec<DynSolValue> = vec![token0_placeholder, token1_address]
                .into_iter()
                .map(DynSolValue::from)
                .collect();
            let to = Address::from_str(&wallet_address)?; // Use wallet address from .env
            let deadline = U256::from(
                (std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)?
                    .as_secs()
                    + 5 * 60) as u64,
            );

            let callbuilder = router_contract
                .function(
                    "swapExactTokensForTokensSupportingFeeOnTransferTokens",
                    &[
                        DynSolValue::from(amount_in),
                        DynSolValue::from(amount_out_min),
                        DynSolValue::from(path),
                        DynSolValue::from(to),
                        DynSolValue::from(deadline),
                    ],
                )?
                .nonce(24u64)
                .chain_id(8453u64)
                .gas(300_000u64)
                .gas_price(20_000_000u128)
                .max_fee_per_gas(20_000_000_000u128)
                .max_priority_fee_per_gas(1_000_000_000u128);

            let tx_hash = callbuilder
                .send()
                .await?;

            println!("Transaction submitted: {:?}", tx_hash);
        } else {
            println!("Failed to extract a valid token1 address");
        }
    }
    Ok(())
}