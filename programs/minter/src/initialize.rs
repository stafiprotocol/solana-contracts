use anchor_lang::prelude::*;
use anchor_spl::token::Mint;

pub use crate::errors::Errors;
pub use crate::states::*;
pub use crate::ID;

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(zero)]
    pub minter: Box<Account<'info, Minter>>,

    pub token_mint: Box<Account<'info, Mint>>,

    pub admin: Signer<'info>,

    pub rent: Sysvar<'info, Rent>,
}

impl<'info> Initialize<'info> {
    pub fn minter_address(&self) -> &Pubkey {
        self.minter.to_account_info().key
    }

    pub fn find_token_mint_authority(minter: &Pubkey) -> (Pubkey, u8) {
        Pubkey::find_program_address(
            &[&minter.to_bytes()[..32], Minter::MINT_AUTHORITY_SEED],
            &ID,
        )
    }

    fn check_token_mint(&mut self) -> Result<u8> {
        let (authority_address, authority_bump) =
            Initialize::find_token_mint_authority(self.minter_address());

        msg!(
            "authority address{}, bump {}",
            authority_address,
            authority_bump
        );

        if !self.token_mint.freeze_authority.is_none() {
            return err!(Errors::InvalidTokenAccountData);
        }
        Ok(authority_bump)
    }

    pub fn process(&mut self, ext_mint_authorities: Vec<Pubkey>) -> Result<()> {
        let token_mint_authority_seed_bump = self.check_token_mint()?;
        self.minter.set_inner(Minter {
            admin: self.admin.key(),
            token_mint: self.token_mint.key(),
            mint_authority_seed_bump: token_mint_authority_seed_bump,
            ext_mint_authorities,
        });
        Ok(())
    }
}
