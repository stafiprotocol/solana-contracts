use anchor_lang::prelude::*;

#[account]
#[derive(Debug)]
pub struct MintManager {
    pub admin: Pubkey,
    pub rsol_mint: Pubkey,
    pub mint_authority_seed_bump: u8,
    pub ext_mint_authorities: Vec<Pubkey>,
}

impl MintManager {
    pub const MINT_AUTHORITY_SEED: &'static [u8] = b"mint";
}
