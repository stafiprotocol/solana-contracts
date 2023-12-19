use anchor_lang::prelude::*;
use anchor_spl::token::Mint;

pub use crate::errors::Errors;
pub use crate::states::*;
pub use crate::ID;

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(zero)]
    pub mint_manager: Box<Account<'info, MintManagerAccount>>,

    /// CHECK: pda
    #[account(
        seeds = [
            &mint_manager.key().to_bytes(),
            MintManagerAccount::MINT_AUTHORITY_SEED,
        ],
        bump,
    )]
    pub mint_authority: UncheckedAccount<'info>,

    pub rsol_mint: Box<Account<'info, Mint>>,

    pub admin: Signer<'info>,

    pub rent: Sysvar<'info, Rent>,
}

impl<'info> Initialize<'info> {
    pub fn process(
        &mut self,
        ext_mint_authorities: Vec<Pubkey>,
        mint_authority_seed_bump: u8,
    ) -> Result<()> {
        require!(
            self.rsol_mint.freeze_authority.is_none(),
            Errors::InvalidTokenAccountData
        );

        self.mint_manager.set_inner(MintManagerAccount {
            admin: self.admin.key(),
            rsol_mint: self.rsol_mint.key(),
            mint_authority_seed_bump,
            ext_mint_authorities,
        });
        Ok(())
    }
}
