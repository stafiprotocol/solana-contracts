use crate::{Errors, StakeManager};
use anchor_lang::prelude::*;
use anchor_lang::solana_program::system_program;
use anchor_lang::system_program::{transfer, Transfer};
use anchor_spl::token::{Mint, Token, TokenAccount};

use mint_manager_program::cpi::accounts::MintToken;
use mint_manager_program::program::MintManagerProgram;
use mint_manager_program::{self, MintManager};

#[derive(Accounts)]
pub struct Stake<'info> {
    #[account(
        mut,
        has_one = rsol_mint @Errors::MintAccountNotMatch,
    )]
    pub stake_manager: Box<Account<'info, StakeManager>>,

    #[account(
        mut,
        seeds = [
            &stake_manager.key().to_bytes(),
            StakeManager::POOL_SEED,
        ],
        bump = stake_manager.pool_seed_bump
    )]
    pub stake_pool: SystemAccount<'info>,

    #[account(
        mut,
        owner = system_program::ID,
        address = mint_to.owner @ Errors::MintToOwnerNotMatch
    )]
    pub from: Signer<'info>,

    #[account(
        has_one = rsol_mint @Errors::MintAccountNotMatch
    )]
    pub mint_manager: Box<Account<'info, MintManager>>,

    #[account(mut)]
    pub rsol_mint: Box<Account<'info, Mint>>,

    #[account(
        mut,
        token::mint = stake_manager.rsol_mint
    )]
    pub mint_to: Box<Account<'info, TokenAccount>>,

    /// CHECK:  check on mint manager program
    pub mint_authority: UncheckedAccount<'info>,

    pub mint_manager_program: Program<'info, MintManagerProgram>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
}

#[event]
pub struct EventStake {
    pub era: u64,
    pub staker: Pubkey,
    pub mint_to: Pubkey,
    pub stake_amount: u64,
    pub rsol_amount: u64,
}

impl<'info> Stake<'info> {
    pub fn process(&mut self, stake_amount: u64) -> Result<()> {
        require_gte!(
            stake_amount,
            self.stake_manager.min_stake_amount,
            Errors::StakeAmountTooLow
        );

        let user_balance = self.from.lamports();
        require_gte!(user_balance, stake_amount, Errors::BalanceNotEnough);

        let rsol_amount = self.stake_manager.calc_rsol_amount(stake_amount)?;

        self.stake_manager.era_bond += stake_amount;
        self.stake_manager.active += stake_amount;

        // transfer lamports to the pool
        transfer(
            CpiContext::new(
                self.system_program.to_account_info(),
                Transfer {
                    from: self.from.to_account_info(),
                    to: self.stake_pool.to_account_info(),
                },
            ),
            stake_amount,
        )?;

        // mint rsol
        let cpi_program = self.mint_manager_program.to_account_info();
        let cpi_accounts = MintToken {
            mint_manager: self.mint_manager.to_account_info(),
            rsol_mint: self.rsol_mint.to_account_info(),
            mint_to: self.mint_to.to_account_info(),
            mint_authority: self.mint_authority.to_account_info(),
            ext_mint_authority: self.stake_pool.to_account_info(),
            token_program: self.token_program.to_account_info(),
        };
        mint_manager_program::cpi::mint_token(
            CpiContext::new(cpi_program, cpi_accounts).with_signer(&[&[
                &self.stake_manager.key().to_bytes(),
                StakeManager::POOL_SEED,
                &[self.stake_manager.pool_seed_bump],
            ]]),
            rsol_amount,
        )?;

        self.stake_manager.total_rsol_supply += rsol_amount;

        emit!(EventStake {
            era: self.stake_manager.latest_era,
            staker: self.from.key(),
            mint_to: self.mint_to.key(),
            stake_amount,
            rsol_amount
        });
        Ok(())
    }
}
