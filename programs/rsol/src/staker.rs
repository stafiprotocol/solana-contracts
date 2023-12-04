use crate::{Errors, StakeManager, UnstakeAccount};
use anchor_lang::prelude::*;
use anchor_lang::solana_program::system_program;
use anchor_lang::system_program::{transfer, Transfer};
use anchor_spl::token::{burn, Burn, Mint, Token, TokenAccount};
use minter::cpi::accounts::MintToken;
use minter::program::Minter;
use minter::{self, MintManager};

#[derive(Accounts)]
pub struct Stake<'info> {
    #[account(mut)]
    pub stake_manager: Account<'info, StakeManager>,

    #[account(
        mut,
        owner = system_program::ID
    )]
    pub from: Signer<'info>,

    #[account(mut)]
    pub target_pool: SystemAccount<'info>,

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

    /// CHECK: pda
    #[account(
        seeds = [
            &stake_manager.key().to_bytes(),
            StakeManager::EXT_MINT_AUTHORITY_SEED
        ],
        bump = stake_manager.ext_mint_authority_seed_bump
    )]
    pub ext_mint_authority: UncheckedAccount<'info>,

    pub minter_program: Program<'info, Minter>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
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

        let pool = self
            .stake_manager
            .bonded_pools
            .get_mut(&self.target_pool.key())
            .ok_or_else(|| error!(Errors::PoolNotExist))?;

        pool.bond = pool.bond + stake_amount;
        pool.active = pool.active + stake_amount;

        // transfer lamports to the pool
        transfer(
            CpiContext::new(
                self.system_program.to_account_info(),
                Transfer {
                    from: self.from.to_account_info(),
                    to: self.target_pool.to_account_info(),
                },
            ),
            stake_amount,
        )?;

        // mint rsol
        let cpi_program = self.minter_program.to_account_info();
        let cpi_accounts = MintToken {
            mint_manager: self.mint_manager.to_account_info(),
            rsol_mint: self.rsol_mint.to_account_info(),
            mint_to: self.mint_to.to_account_info(),
            mint_authority: self.mint_authority.to_account_info(),
            ext_mint_authority: self.ext_mint_authority.to_account_info(),
            token_program: self.token_program.to_account_info(),
        };
        minter::cpi::mint_token(
            CpiContext::new(cpi_program, cpi_accounts).with_signer(&[&[][..]]),
            rsol_amount,
        )?;

        Ok(())
    }
}

#[derive(Accounts)]
pub struct Unstake<'info> {
    #[account(mut)]
    pub stake_manager: Box<Account<'info, StakeManager>>,

    #[account(mut)]
    pub rsol_mint: Box<Account<'info, Mint>>,

    #[account(
        mut,
        token::mint = stake_manager.rsol_mint
    )]
    pub burn_from: Box<Account<'info, TokenAccount>>,

    pub burn_authority: Signer<'info>,

    #[account(
        zero,
        rent_exempt = enforce
    )]
    pub unstake_account: Box<Account<'info, UnstakeAccount>>,

    pub clock: Sysvar<'info, Clock>,
    pub rent: Sysvar<'info, Rent>,
    pub token_program: Program<'info, Token>,
}

impl<'info> Unstake<'info> {
    pub fn process(&mut self, unstake_amount: u64) -> Result<()> {
        check_token_account(&self.burn_from, self.burn_authority.key, unstake_amount)?;

        let sol_amount = self.stake_manager.calc_sol_amount(unstake_amount)?;

        burn(
            CpiContext::new(
                self.token_program.to_account_info(),
                Burn {
                    mint: self.rsol_mint.to_account_info(),
                    from: self.burn_from.to_account_info(),
                    authority: self.burn_authority.to_account_info(),
                },
            ),
            unstake_amount,
        )?;

        self.unstake_account.set_inner(UnstakeAccount {
            stake_manager: self.stake_manager.key(),
            recipient: self.burn_from.owner,
            amount: sol_amount,
            created_epoch: self.clock.epoch,
        });

        Ok(())
    }
}

#[macro_export]
macro_rules! require_lte {
    ($value1: expr, $value2: expr, $error_code: expr $(,)?) => {
        if $value1 > $value2 {
            return Err(error!($error_code).with_values(($value1, $value2)));
        }
    };
}

#[macro_export]
macro_rules! require_lt {
    ($value1: expr, $value2: expr, $error_code: expr $(,)?) => {
        if $value1 >= $value2 {
            return Err(error!($error_code).with_values(($value1, $value2)));
        }
    };
}

pub fn check_token_account<'info>(
    token_account: &Account<'info, TokenAccount>,
    authority: &Pubkey,
    token_amount: u64,
) -> Result<()> {
    if token_account.delegate.contains(authority) {
        require_lte!(
            token_amount,
            token_account.delegated_amount,
            Errors::BalanceNotEnough
        );
    } else if *authority == token_account.owner {
        require_lte!(token_amount, token_account.amount, Errors::BalanceNotEnough);
    } else {
        return err!(Errors::AuthorityNotMatch);
    }
    Ok(())
}
