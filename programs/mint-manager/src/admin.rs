pub use crate::errors::Errors;
pub use crate::states::*;
use anchor_lang::prelude::*;

#[derive(Accounts)]
pub struct TransferAdmin<'info> {
    #[account(
        mut,
        has_one = admin @ Errors::AdminNotMatch
    )]
    pub mint_manager: Account<'info, MintManager>,

    pub admin: Signer<'info>,
}

impl<'info> TransferAdmin<'info> {
    pub fn process(&mut self, new_admin: Pubkey) -> Result<()> {
        self.mint_manager.admin = new_admin;
        Ok(())
    }
}

#[derive(Accounts)]
pub struct SetExtMintAuthorities<'info> {
    #[account(
        mut, 
        has_one = admin @ Errors::AdminNotMatch
    )]
    pub mint_manager: Box<Account<'info, MintManager>>,

    pub admin: Signer<'info>,
}

impl<'info> SetExtMintAuthorities<'info> {
    pub fn process(&mut self, ext_mint_authorities: Vec<Pubkey>) -> Result<()> {
        self.mint_manager.ext_mint_authorities = ext_mint_authorities;
        Ok(())
    }
}
