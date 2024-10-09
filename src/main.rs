use std::io::{self, Write};

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
    let (metadata_rest, mintdata_res) = tokio::join!(
        fetch_token_metadata(&client, &token_address),
        fetch_token_mintdata(&client, &token_address),
    );

    let (token_name, token_symbol, offchain_data, dns_records) =
        metadata_rest.with_context(|| "Failed to retrieve token metadata")?;

    let mintdata = mintdata_res.with_context(|| "Failed to retrieve token mint data")?;

    let stdout = io::stdout();
    let mut handle = io::BufWriter::new(stdout);

    // Output token information
    writeln!(handle, "Token Name: {}", token_name)?;
    writeln!(handle, "Token Symbol: {}", token_symbol)?;
    writeln!(handle, "Total Supply: {}", mintdata.supply)?;
    writeln!(handle, "Decimals: {}", mintdata.decimals)?;
    writeln!(
        handle,
        "Mint Authority: {}",
        pubkey_to_string(mintdata.mint_authority)
    )?;

    writeln!(
        handle,
        "Freeze Authority: {}",
        pubkey_to_string(mintdata.freeze_authority)
    )?;

    writeln!(
        handle,
        "Token Description: {}",
        offchain_data.description.unwrap_or(UNAVAILABLE.to_string())
    )?;
    writeln!(
        handle,
        "Token Image: {}",
        offchain_data.image.unwrap_or(UNAVAILABLE.to_string())
    )?;

    writeln!(
        handle,
        "Token Website: {}",
        offchain_data.website.unwrap_or(UNAVAILABLE.to_string())
    )?;

    writeln!(
        handle,
        "Number of DNS records: {}",
        dns_records.unwrap_or(UNAVAILABLE.to_string())
    )?;

    Ok(())
}

/*
9KgvborfMPc1nzhXe9N8Q9pKTt57YdBWt9VqHnibdqjC
 */
