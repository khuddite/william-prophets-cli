use std::{
    io::{self, Write},
    time::Duration,
};

use anyhow::{Context, Result};
use clap::Parser;
use indicatif::ProgressBar;
use prophetbots_cli::*;
use solana_client::nonblocking::rpc_client::RpcClient;

#[tokio::main]
async fn main() -> Result<()> {
    let cli = ProphetsCli::parse();
    let token_address = cli.token_address;

    // Load config
    let cfg = get_config().with_context(|| "Unable to load CLI config")?;
    let rpc_url = cfg.rpc_url();

    // Create Solana RPC client
    let client = RpcClient::new(rpc_url.to_string());

    // Set up a spinner
    let bar = ProgressBar::new_spinner();
    bar.enable_steady_tick(Duration::from_millis(100));
    bar.set_message("Fetching token details...");

    // Fetch token information concurrently
    let (metadata_rest, mintdata_res) = tokio::join!(
        fetch_token_metadata(&client, &token_address),
        fetch_token_mintdata(&client, &token_address),
    );

    bar.finish();

    let (token_name, token_symbol, offchain_data, dns_records) =
        metadata_rest.with_context(|| {
            "Failed to retrieve token metadata, it's likely because the token address is invalid"
        })?;

    let mintdata = mintdata_res.with_context(|| {
        "Failed to retrieve token mint data, it's likely because the token address is invalid"
    })?;

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
        string_or_not_available(offchain_data.description)
    )?;
    writeln!(
        handle,
        "Token Image: {}",
        string_or_not_available(offchain_data.image)
    )?;

    writeln!(
        handle,
        "Token Website: {}",
        string_or_not_available(offchain_data.website)
    )?;

    writeln!(
        handle,
        "Number of DNS records: {}",
        string_or_not_available(dns_records)
    )?;

    Ok(())
}

/*
EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v
9KgvborfMPc1nzhXe9N8Q9pKTt57YdBWt9VqHnibdqjC
 */
