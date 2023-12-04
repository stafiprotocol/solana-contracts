use anchor_lang::prelude::*;
use anchor_spl::token::{mint_to, Mint, MintTo, Token, TokenAccount};

pub use crate::errors::Errors;
pub use crate::states::*;

#[derive(Accounts)]
pub struct MintToken<'info> {
    pub mint_manager: Box<Account<'info, MintManager>>,

    #[account(mut)]
    pub rsol_mint: Box<Account<'info, Mint>>,

    #[account(
        mut,
        token::mint = mint_manager.rsol_mint
    )]
    pub mint_to: Box<Account<'info, TokenAccount>>,

    /// CHECK: pda
    #[account(
        seeds = [
            &mint_manager.key().to_bytes(),
            MintManager::MINT_AUTHORITY_SEED
        ],
        bump = mint_manager.mint_authority_seed_bump
    )]
    pub mint_authority: UncheckedAccount<'info>,

    pub ext_mint_authority: Signer<'info>,

    pub token_program: Program<'info, Token>,
}

impl<'info> MintToken<'info> {
    pub fn process(&mut self, mint_amount: u64) -> Result<()> {
        if !self
            .mint_manager
            .ext_mint_authorities
            .contains(self.ext_mint_authority.key)
        {
            return err!(Errors::InvalidExtMintAuthority);
        }

        mint_to(
            CpiContext::new_with_signer(
                self.token_program.to_account_info(),
                MintTo {
                    mint: self.rsol_mint.to_account_info(),
                    to: self.mint_to.to_account_info(),
                    authority: self.mint_authority.to_account_info(),
                },
                &[&[
                    &self.mint_manager.key().to_bytes(),
                    MintManager::MINT_AUTHORITY_SEED,
                    &[self.mint_manager.mint_authority_seed_bump],
                ]],
            ),
            mint_amount,
        )?;

        msg!("rsol mint amount: {}", mint_amount);

        Ok(())
    }
}
