use anchor_lang::prelude::*;
use anchor_spl::token::{mint_to, Mint, MintTo, Token, TokenAccount};

pub use crate::errors::Errors;
pub use crate::states::*;

#[derive(Accounts)]
pub struct MintToken<'info> {
    pub minter: Box<Account<'info, Minter>>,

    #[account(mut)]
    pub token_mint: Box<Account<'info, Mint>>,

    #[account(
        mut,
        token::mint = minter.token_mint
    )]
    pub mint_to: Box<Account<'info, TokenAccount>>,

    /// CHECK: pda
    #[account(
        seeds = [
            &minter.key().to_bytes(),
            Minter::MINT_AUTHORITY_SEED
        ],
        bump = minter.mint_authority_seed_bump
    )]
    pub token_mint_authority: UncheckedAccount<'info>,

    pub ext_mint_authority: Signer<'info>,

    pub token_program: Program<'info, Token>,
}

impl<'info> MintToken<'info> {
    pub fn process(&mut self, mint_amount: u64) -> Result<()> {
        if !self
            .minter
            .ext_mint_authorities
            .contains(self.ext_mint_authority.key)
        {
            return err!(Errors::InvalidExtMintAuthority);
        }

        mint_to(
            CpiContext::new_with_signer(
                self.token_program.to_account_info(),
                MintTo {
                    mint: self.token_mint.to_account_info(),
                    to: self.mint_to.to_account_info(),
                    authority: self.token_mint_authority.to_account_info(),
                },
                &[&[
                    &self.minter.key().to_bytes(),
                    Minter::MINT_AUTHORITY_SEED,
                    &[self.minter.mint_authority_seed_bump],
                ]],
            ),
            mint_amount,
        )?;

        msg!("token mint {}", mint_amount);

        Ok(())
    }
}
