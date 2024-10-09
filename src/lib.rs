use clap::Parser;
use mpl_token_metadata::accounts::Metadata;
use mpl_token_metadata::ID as METAPLEX_PROGRAM_ID;
use serde::Deserialize;
use solana_client::rpc_client::RpcClient;
use spl_token::solana_program::program_pack::Pack;
use spl_token::solana_program::pubkey::Pubkey;
use spl_token::state::Mint;
use trust_dns_resolver::config::{ResolverConfig, ResolverOpts};
use trust_dns_resolver::TokioAsyncResolver;

const METADATA_SEED: &[u8; 8] = b"metadata";

#[derive(Parser)]
#[command(about = "Fetch on/off chain token details", long_about = None)]
pub struct TokenCli {
    /// Solana token mint address
    pub token_address: Pubkey,
}

#[derive(Deserialize, Debug)]
pub struct OffChainMetadata {
    name: String,
    symbol: String,
    #[serde(rename = "external_url")]
    website: Option<String>,
}

impl OffChainMetadata {
    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn symbol(&self) -> &str {
        &self.symbol
    }

    pub fn website(&self) -> &Option<String> {
        &self.website
    }
}

// Function to get Metaplex metadata PDA
fn get_metadata_pda(mint: &Pubkey) -> Pubkey {
    let metadata_seeds = &[METADATA_SEED, METAPLEX_PROGRAM_ID.as_ref(), &mint.as_ref()];
    let metadata_pubkey = Pubkey::new_from_array(METAPLEX_PROGRAM_ID.to_bytes());

    Pubkey::find_program_address(metadata_seeds, &metadata_pubkey).0
}

// Function to get token metadata from Solana
pub async fn fetch_token_metadata(
    mint_pubkey: &Pubkey,
) -> Result<Metadata, Box<dyn std::error::Error>> {
    let rpc_url: &str = "https://api.mainnet-beta.solana.com";
    let client = RpcClient::new(rpc_url.to_string());
    let metadata_pubkey = get_metadata_pda(&mint_pubkey);
    let account_data = client.get_account_data(&metadata_pubkey)?;
    let metadata = Metadata::from_bytes(&account_data)?;
    Ok(metadata)
}

// Function to fetch token total supply
pub async fn fetch_token_supply(mint_pubkey: &Pubkey) -> Result<u64, Box<dyn std::error::Error>> {
    let rpc_url = "https://api.mainnet-beta.solana.com";
    let client = RpcClient::new(rpc_url.to_string());

    let account_data = client.get_account_data(&mint_pubkey)?;
    let mint_info = Mint::unpack(&account_data)?;
    Ok(mint_info.supply)
}

// Function to fetch off-chain metadata from a URI
pub async fn fetch_off_chain_metadata(
    uri: &str,
) -> Result<OffChainMetadata, Box<dyn std::error::Error>> {
    let response = reqwest::get(uri).await?;
    let metadata: OffChainMetadata = response.json().await?;
    Ok(metadata)
}

// Function to fetch number of DNS records from a website
pub async fn fetch_dns_records(website: &str) -> Result<usize, Box<dyn std::error::Error>> {
    let resolver = TokioAsyncResolver::tokio(ResolverConfig::default(), ResolverOpts::default());
    let response = resolver.lookup_ip("sollamas.com").await?;
    Ok(response.iter().count())
}
