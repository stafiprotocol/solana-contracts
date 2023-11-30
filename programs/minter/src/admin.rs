pub use crate::errors::Errors;
pub use crate::states::*;
use anchor_lang::prelude::*;

#[derive(Accounts)]
pub struct TransferAdmin<'info> {
    #[account(mut,has_one = admin @ Errors::AdminNotMatch)]
    pub minter: Account<'info, Minter>,
    
    pub admin: Signer<'info>,
}

impl<'info> TransferAdmin<'info> {
    pub fn process(&mut self, new_admin: Pubkey) -> Result<()> {
        self.minter.admin = new_admin;
        Ok(())
    }
}

#[derive(Accounts)]
pub struct SetMintAuthorities<'info> {
    #[account(mut, has_one = admin @ Errors::AdminNotMatch)]
    pub minter: Box<Account<'info, Minter>>,

    pub admin: Signer<'info>,
}

impl<'info> SetMintAuthorities<'info> {
    pub fn process(&mut self, ext_mint_authorities: Vec<Pubkey>) -> Result<()> {
        self.minter.ext_mint_authorities = ext_mint_authorities;
        Ok(())
    }
}