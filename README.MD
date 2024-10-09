# Prophets CLI by William Hall

## 1. Problem to solve

### Build a CLI tool that enables a user to input a Solana token address. The program should then fetch and return the following information as quickly as possible

- Token Name
- Token Symbol
- Total Supply
- Website
- Number of Website DNS Records

### We want to see how you handle

- Error Handling: Gracefully manage errors, ensuring the user always gets meaningful feedback, even when something goes wrong.
- Concurrency & Parallelism: Make use of Rust's powerful async capabilities, particularly with Tokio. Fetch the information concurrently to optimize speed.
- Code Quality: Write clean, succinct code. Show us your best practices, from structuring your project to writing clear and useful documentation.

## 2. Solution

> This solution is designed to support both fungible and non-fungible tokens, displaying "Not available" for any missing data.

### Crates Used

```
- clap: Parses command-line arguments.
- mpl-token-metadata: Fetches Metaplex metadata from the Solana blockchain.
- reqwest: Retrieves off-chain metadata via HTTP requests.
- serde: Serializes and deserializes off-chain metadata in JSON format.
- solana-client: Establishes RPC connections with the Solana network.
- spl-token: Parses raw Solana account data into the Mint struct.
- tokio: Provides asynchronous concurrency support.
- trust-dns-resolver: Fetches the number of DNS records associated with the token's website.
- anyhow: Simplifies error handling for efficient and readable code.
- confy: Manages the RPC URL configuration for scalability and flexibility.
- indicatif: Displays progress bars for enhanced user experience.
- url: Extracts the domain from the website URL.
- assert_cmd, predicates: Implements integration testing.
```

---

### Structure

```
- main.rs: Manages command-line input and output, and invokes business logic from lib.rs.
- lib.rs: Contains core logic, functions, and structs for fetching token details.
```

The token's supply, mint authority, and freeze authority are stored within the Solana blockchain in the mint account data. These are retrieved via RPC calls and parsed into the Mint struct using the spl-token crate.

Token metadata, including the name, symbol, and URI, is managed by the Metaplex program on Solana. This metadata is stored in a Metaplex Program Derived Address (PDA) and parsed into the Metadata struct from the mpl-token-metadata crate. The URI links to off-chain metadata, which includes additional information such as the token's description, website URL, and image. Fetching the Metaplex metadata, retrieving off-chain metadata, and fetching DNS records are done sequentially.

---

### Testing/Documentation

Both unit and integration tests have been implemented for all relevant functions and scenarios, with comments added to improve readability and maintainability.

---

### Example Outputs

- USD Coin
  ```
  Token Name: USD Coin
  Token Symbol: USDC
  Total Supply: 2744153149184729
  Decimals: 6
  Mint Authority: BJE5MMbqXjVwjAF7oxwPYXnTXDyspzZyt4vwenNw5ruG
  Freeze Authority: 7dGbd2QZcCKcTndnHcTL8q7SMVXAkp688NTQYwrRCrar
  Token Description: Not available
  Token Image: Not available
  Token Website: Not available
  Number of DNS records: Not available
  ```

- Sollamas
  ```
  Token Name: Alessio, the Pleasant
  Token Symbol: LLAMA
  Total Supply: 1
  Decimals: 0
  Mint Authority: EBqKJuEN84EZUyXexGmgZvxNc3uetMNbkTsQq8ArKbYU
  Freeze Authority: EBqKJuEN84EZUyXexGmgZvxNc3uetMNbkTsQq8ArKbYU
  Token Description: Alessio, the Pleasant is a uniquely generated, cute and collectible Llama with proof of ownership on the Solana blockchain. He was #4,736 to be minted!
  Token Image: https://arweave.net/IflrmClNlH_wXG_XdfZNfqvwEsmesWRbHtkGC87-WPI
  Token Website: https://sollamas.com
  Number of DNS records: 4
  ```
  
---