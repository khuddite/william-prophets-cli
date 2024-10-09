use anyhow::{Context, Result};
use clap::Parser;
use mpl_token_metadata::accounts::Metadata;
use mpl_token_metadata::ID as METAPLEX_PROGRAM_ID;
use serde::{Deserialize, Serialize};
use solana_client::nonblocking::rpc_client::RpcClient;
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
pub struct ProphetsCli {
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
    let metadata_seeds = &[METADATA_SEED, METAPLEX_PROGRAM_ID.as_ref(), (mint.as_ref())];

    // Get a metaplex pubkey
    let metaplex_pubkey = Pubkey::new_from_array(METAPLEX_PROGRAM_ID.to_bytes());

    Pubkey::find_program_address(metadata_seeds, &metaplex_pubkey).0
}

// Function to get token metadata from Solana
async fn fetch_on_chain_metadata(client: &RpcClient, mint_pubkey: &Pubkey) -> Result<Metadata> {
    // Get Metaplex PDA address from mint account address
    let metadata_pubkey = get_metadata_pda(mint_pubkey);

    // Fetch on-chain metadata
    let account_data = client
        .get_account_data(&metadata_pubkey)
        .await
        .with_context(|| "Failed to load Metaplex metadata")?;

    // Parse raw on-chain metadata into Metaplex's Metadata struct
    let metadata =
        Metadata::from_bytes(&account_data).with_context(|| "Failed to parse Metaplex metadata")?;

    Ok(metadata)
}

// Function to fetch off-chain metadata from a URI
async fn fetch_off_chain_metadata(uri: &str) -> Result<OffChainMetadata> {
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

    Ok(offchain_metadata)
}

// Function to fetch number of DNS records from a website
async fn fetch_dns_records(website: &Option<String>) -> Option<String> {
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
        .get_account_data(mint_pubkey)
        .await
        .with_context(|| "Failed to load mint account data")?;

    // Parse mint account data into Mint struct
    let mint_info =
        Mint::unpack(&account_data).with_context(|| "Failed to parse mint account data")?;

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
        metadata.name.trim_end_matches(char::from(0)).to_string(),
        metadata.symbol.trim_end_matches(char::from(0)).to_string(),
        offchain_metadata,
        dns_records,
    ))
}

pub fn pubkey_to_string(pubkey: COption<Pubkey>) -> String {
    if let COption::Some(pubkey) = pubkey {
        pubkey.to_string()
    } else {
        UNAVAILABLE.to_string()
    }
}

pub fn string_or_not_available(info_str: Option<String>) -> String {
    if let Some(info_str) = info_str {
        if !info_str.is_empty() {
            return info_str;
        }
    }

    UNAVAILABLE.to_string()
}

#[cfg(test)]
mod cli_tests {
    use std::str::FromStr;

    use super::*;

    #[test]
    fn string_or_not_available_test() {
        let result = string_or_not_available(None);
        assert_eq!(result, UNAVAILABLE.to_string());

        let result = string_or_not_available(Some("".to_string()));
        assert_eq!(result, UNAVAILABLE.to_string());

        let test_string = String::from("test");
        let result = string_or_not_available(Some(test_string.clone()));

        assert_eq!(result, test_string);
    }

    #[test]
    fn pubkey_to_string_test() {
        let result = pubkey_to_string(COption::None);
        assert_eq!(result, UNAVAILABLE.to_string());

        let test_pubkey = Pubkey::new_unique();
        let result = pubkey_to_string(COption::Some(test_pubkey));
        assert_eq!(result, test_pubkey.to_string());
    }

    #[tokio::test]
    async fn fetch_token_mintdata_test() {
        let client = RpcClient::new("https://api.mainnet-beta.solana.com".to_string());

        // USDC mint account
        let test_mint_pubkey =
            Pubkey::from_str("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v").unwrap();

        let result = fetch_token_mintdata(&client, &test_mint_pubkey).await;

        assert!(result.is_ok());

        let result = result.unwrap();
        assert_eq!(result.decimals, 6);
        assert!(result.is_initialized);

        // non-mint account
        let test_mint_pubkey =
            Pubkey::from_str("BJE5MMbqXjVwjAF7oxwPYXnTXDyspzZyt4vwenNw5ruG").unwrap();
        let result = fetch_token_mintdata(&client, &test_mint_pubkey).await;

        assert!(result.is_err());

        // NFT mint account
        let test_mint_pubkey =
            Pubkey::from_str("7fxxyaTv3Y19Coc1kwwaniDSHNboNqHTYvVvtMxr1uWo").unwrap();

        let result = fetch_token_mintdata(&client, &test_mint_pubkey).await;

        assert!(result.is_ok());
        let result = result.unwrap();
        assert_eq!(result.decimals, 0);
        assert_eq!(result.supply, 1);
        assert!(result.is_initialized);
    }

    #[tokio::test]
    async fn fetch_token_metadata_test() {
        let client = RpcClient::new("https://api.mainnet-beta.solana.com".to_string());

        // USDC mint account
        let test_mint_pubkey =
            Pubkey::from_str("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v").unwrap();

        let result = fetch_token_metadata(&client, &test_mint_pubkey).await;

        assert!(result.is_ok());

        let (name, symbol, offchain_metadata, dns_records) = result.unwrap();
        assert_eq!(name, "USD Coin");
        assert_eq!(symbol, "USDC");
        assert_eq!(offchain_metadata.description, None);
        assert_eq!(offchain_metadata.image, None);
        assert_eq!(offchain_metadata.website, None);
        assert_eq!(dns_records, None);

        // non-mint account
        let test_mint_pubkey =
            Pubkey::from_str("BJE5MMbqXjVwjAF7oxwPYXnTXDyspzZyt4vwenNw5ruG").unwrap();
        let result = fetch_token_metadata(&client, &test_mint_pubkey).await;

        assert!(result.is_err());

        // NFT mint account
        let test_mint_pubkey =
            Pubkey::from_str("7fxxyaTv3Y19Coc1kwwaniDSHNboNqHTYvVvtMxr1uWo").unwrap();

        let result: std::result::Result<
            (String, String, OffChainMetadata, Option<String>),
            anyhow::Error,
        > = fetch_token_metadata(&client, &test_mint_pubkey).await;

        assert!(result.is_ok());
        let (name, symbol, offchain_metadata, dns_records) = result.unwrap();
        assert_eq!(name, "Signal Boost #088");
        assert_eq!(symbol, "SGBT2");
        assert_eq!(offchain_metadata.description, Some("Signal Boost is a 3D art collection by Jack Dupp. It is an exploration of color and light through a process of 3D extrapolation of a 2D artwork. The original artwork is permanently destroyed revealing a new energetic outcome.".to_string()));
        assert_eq!(
            offchain_metadata.website,
            Some("https://abstractlabs.art".to_string())
        );
        assert_eq!(
            offchain_metadata.image,
            Some(
                "https://www.arweave.net/eY9gWuLKyBRsNv30Xug79GzjfiW4DJ2xfoFMa1-RZ8A?ext=jpg"
                    .to_string()
            )
        );
        assert_eq!(dns_records, None);
    }
}
