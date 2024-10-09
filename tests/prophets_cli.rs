use anyhow::Result;
use assert_cmd::prelude::*; // Add methods on commands
use predicates::prelude::*; // Used for writing assertions
use std::process::Command; // Run programs

#[test]
fn invalid_mint_address() -> Result<()> {
    let mut cmd = Command::cargo_bin("prophetbots-cli")?;

    // Invalid mint account address
    cmd.arg("BJE5MMbqXjVwjAF7oxwPYXnTXDyspzZyt4vwenNw5ruG");
    cmd.assert().failure().stderr(predicate::str::contains(
        "it\'s likely because the token address is invalid",
    ));

    Ok(())
}

#[test]
fn invalid_solana_address() -> Result<()> {
    let mut cmd = Command::cargo_bin("prophetbots-cli")?;

    // Invalid solana account address
    cmd.arg("asdf");
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("invalid value"));

    Ok(())
}

#[test]
fn output_ft_details() -> Result<()> {
    let mut cmd = Command::cargo_bin("prophetbots-cli")?;

    // USDC mint account address
    cmd.arg("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Token Name: USD Coin"))
        .stdout(predicate::str::contains("Token Symbol: USDC"))
        .stdout(predicate::str::contains("Decimals: 6"))
        .stdout(predicate::str::contains("Token Description: Not available"))
        .stdout(predicate::str::contains("Token Image: Not available"))
        .stdout(predicate::str::contains(
            "Number of DNS records: Not available",
        ));

    Ok(())
}

#[test]
fn output_nft_details() -> Result<()> {
    let mut cmd = Command::cargo_bin("prophetbots-cli")?;

    // NFT mint account address
    cmd.arg("9KgvborfMPc1nzhXe9N8Q9pKTt57YdBWt9VqHnibdqjC");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains(
            "Token Name: Alessio, the Pleasant",
        ))
        .stdout(predicate::str::contains("Token Symbol: LLAMA"))
        .stdout(predicate::str::contains("Total Supply: 1"))
        .stdout(predicate::str::contains("Decimals: 0"))
        .stdout(predicate::str::contains("Token Description: Alessio, the Pleasant is a uniquely generated, cute and collectible Llama with proof of ownership on the Solana blockchain. He was #4,736 to be minted!"))
        .stdout(predicate::str::contains("Token Image: https://arweave.net/IflrmClNlH_wXG_XdfZNfqvwEsmesWRbHtkGC87-WPI"))
        .stdout(predicate::str::contains("Token Website: https://sollamas.com"))
        .stdout(predicate::str::contains("Number of DNS records: 4"));

    Ok(())
}
