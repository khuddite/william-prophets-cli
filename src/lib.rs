use anyhow::{Context, Result};
use clap::Parser;
use mpl_token_metadata::accounts::Metadata;
use mpl_token_metadata::ID as METAPLEX_PROGRAM_ID;
use serde::{Deserialize, Serialize};
use solana_client::rpc_client::RpcClient;
use spl_token::solana_program::program_option::COption;
use spl_token::solana_program::program_pack::Pack;
use spl_token::solana_program::pubkey::Pubkey;
use spl_token::state::Mint;
use trust_dns_resolver::config::{ResolverConfig, ResolverOpts};
use trust_dns_resolver::TokioAsyncResolver;
use url::Url;

const METADATA_SEED: &[u8; 8] = b"metadata";
pub const UNAVAILABLE: &str = "Not available";

#[derive(Parser)]
#[command(about = "Fetch on/off chain token details", long_about = None)]
pub struct TokenCli {
    /// Solana mint account address
    pub token_address: Pubkey,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
    rpc_url: String,
}

impl Config {
    pub fn rpc_url(&self) -> &str {
        &self.rpc_url
    }
}

impl Default for Config {
    fn default() -> Self {
        Config {
            rpc_url: "https://api.mainnet-beta.solana.com".to_string(),
        }
    }
}

// Function to load config with default values
pub fn get_config() -> Result<Config> {
    let cfg: Config =
        confy::load("solana_token_cli", None).with_context(|| "Failed to load configuration")?;

    Ok(cfg)
}

#[derive(Deserialize, Debug, Default)]
pub struct OffChainMetadata {
    pub description: Option<String>,
    pub image: Option<String>,
    #[serde(rename = "external_url")]
    pub website: Option<String>,
}

// Function to get Metaplex metadata PDA
fn get_metadata_pda(mint: &Pubkey) -> Pubkey {
    // Construct seeds
    let metadata_seeds = &[METADATA_SEED, METAPLEX_PROGRAM_ID.as_ref(), &mint.as_ref()];

    // Get a metaplex pubkey
    let metaplex_pubkey = Pubkey::new_from_array(METAPLEX_PROGRAM_ID.to_bytes());

    Pubkey::find_program_address(metadata_seeds, &metaplex_pubkey).0
}

// Function to get token metadata from Solana
pub async fn fetch_on_chain_metadata(client: &RpcClient, mint_pubkey: &Pubkey) -> Result<Metadata> {
    // Get Metaplex PDA address from mint account address
    let metadata_pubkey = get_metadata_pda(&mint_pubkey);

    // Fetch on-chain metadata
    let account_data = client
        .get_account_data(&metadata_pubkey)
        .with_context(|| "Failed to load Metaplex metadata")?;

    // Parse raw on-chain metadata into Metaplex's Metadata struct
    let metadata = Metadata::from_bytes(&account_data)
        .with_context(|| "Failed to construct Metaplex metadata")?;

    Ok(metadata)
}

// Function to fetch off-chain metadata from a URI
pub async fn fetch_off_chain_metadata(uri: &str) -> Result<OffChainMetadata> {
    let uri = uri.trim_end_matches(char::from(0));
    // Fetch offchain metadata from URI
    let response = reqwest::get(uri)
        .await
        .with_context(|| "Failed to load offchain metadata")?;

    // Parse raw metadata into JSON
    let offchain_metadata: OffChainMetadata = response
        .json()
        .await
        .with_context(|| "Failed to parse offchain metadata into JSON")?;

    println!("{:?}", offchain_metadata);

    Ok(offchain_metadata)
}

// Function to fetch number of DNS records from a website
pub async fn fetch_dns_records(website: &Option<String>) -> Option<String> {
    if let Some(website) = website {
        if let Ok(parsed_url) = Url::parse(website) {
            // Extract domain from website url
            if let Some(domain) = parsed_url.domain() {
                let resolver =
                    TokioAsyncResolver::tokio(ResolverConfig::default(), ResolverOpts::default());

                // Look up Ipv4 & Ipv6 DNS records
                if let Ok(response) = resolver.lookup_ip(domain).await {
                    return Some(response.iter().count().to_string());
                }
            }
        }
    }

    None
}

// Function to fetch token total supply
pub async fn fetch_token_mintdata(client: &RpcClient, mint_pubkey: &Pubkey) -> Result<Mint> {
    // Fetch on-chain mint account data
    let account_data = client
        .get_account_data(&mint_pubkey)
        .with_context(|| "Failed to load mint account data")?;

    // Parse mint account data into Mint struct
    let mint_info = Mint::unpack(&account_data).with_context(|| "Invalid mint account")?;

    Ok(mint_info)
}

pub async fn fetch_token_metadata(
    client: &RpcClient,
    mint_pubkey: &Pubkey,
) -> Result<(String, String, OffChainMetadata, Option<String>)> {
    let metadata = fetch_on_chain_metadata(client, mint_pubkey).await?;

    // Off-chain metadata is depedent of on-chain metadata (uri), thus this should happen sequentially
    let offchain_metadata = fetch_off_chain_metadata(&metadata.uri)
        .await
        .unwrap_or(OffChainMetadata::default());

    let dns_records = fetch_dns_records(&offchain_metadata.website).await;

    Ok((
        metadata.name,
        metadata.symbol,
        offchain_metadata,
        dns_records,
    ))
}

pub fn pubkey_to_string(pubkey: COption<Pubkey>) -> String {
    if pubkey.is_some() {
        pubkey.unwrap().to_string()
    } else {
        UNAVAILABLE.to_string()
    }
}
