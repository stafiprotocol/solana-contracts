# solana-contracts

Liquid staking on solana by StaFi Protocol.

## Programs

| Program | Version | Description |
| --- | --- |--- |
| stake-manager | v0.2.0 | solana liquid staking manager |
| mint-manager | v0.1.0 | rsol minter manager |
| bridge-manager | v0.1.0 | bridge for stafi and solana chain |

## Development

### Environment Setup

```toml
anchor_version = "0.29.0"
solana_version = "1.16.20"

```

1. Install [Rust](https://rustup.rs/).
2. Install the [Solana tools](https://docs.solanalabs.com/cli/install).
3. Install the [Anchor](https://www.anchor-lang.com/docs/installation).

### Build

```sh
anchor build
```
