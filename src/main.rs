use anyhow::{Context, Result};
use clap::Parser;
use prophetbots_cli::*;
use solana_client::rpc_client::RpcClient;

#[tokio::main]
async fn main() -> Result<()> {
    let cli = TokenCli::parse();
    let token_address = cli.token_address;

    // Load config
    let cfg = get_config().with_context(|| "Unable to load CLI config")?;
    let rpc_url = cfg.rpc_url();

    // Create Solana RPC client
    let client = RpcClient::new(rpc_url);

    // Fetch token information concurrently
    let (metadata_res, supply_res) = tokio::join!(
        fetch_token_metadata(&client, &token_address),
        fetch_token_supply(&client, &token_address),
    );

    let metadata = match metadata_res {
        Ok(metadata) => metadata,
        Err(err) => {
            eprintln!("Error fetching metadata: {}", err);
            return Ok(());
        }
    };

    let supply = match supply_res {
        Ok(supply) => supply,
        Err(err) => {
            eprintln!("Error fetching supply: {}", err);
            return Ok(());
        }
    };

    // Fetch off-chain metadata URI and DNS records concurrently
    let metadata_uri = metadata.uri.trim_end_matches(char::from(0));
    let (off_chain_metadata_res, dns_records_res) = tokio::join!(
        fetch_off_chain_metadata(metadata_uri),
        fetch_dns_records(metadata_uri)
    );

    let off_chain_metadata = match off_chain_metadata_res {
        Ok(data) => data,
        Err(err) => {
            eprintln!("Error fetching off-chain metadata: {}", err);
            return Ok(());
        }
    };

    let dns_records = match dns_records_res {
        Ok(records) => records,
        Err(err) => {
            eprintln!("Error fetching DNS records: {}", err);
            return Ok(());
        }
    };

    // Output token information
    println!("Token Name: {}", off_chain_metadata.name());
    println!("Token Symbol: {}", off_chain_metadata.symbol());
    println!("Total Supply: {}", supply);
    if let Some(website) = off_chain_metadata.website() {
        println!("Website: {}", website);
        println!("Number of DNS Records: {}", dns_records);
    } else {
        println!("Website: Not available");
    }

    Ok(())
}

/*
9KgvborfMPc1nzhXe9N8Q9pKTt57YdBWt9VqHnibdqjC
 */
