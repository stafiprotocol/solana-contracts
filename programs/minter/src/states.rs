use anchor_lang::prelude::*;

#[account]
#[derive(Debug)]
pub struct Minter {
    pub admin: Pubkey,
    pub token_mint: Pubkey,
    pub mint_authority_seed_bump: u8,
    pub ext_mint_authorities: Vec<Pubkey>,
}

impl Minter {
    pub const MINT_AUTHORITY_SEED: &'static [u8] = b"mint";
}
