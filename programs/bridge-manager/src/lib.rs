//! stafi solana bridge.
mod errors;
mod states;
mod tx_accounts;

use anchor_lang::prelude::*;
use anchor_spl::token::{self, spl_token::instruction::AuthorityType};
use anchor_spl::token::{burn, Burn};
use mint_manager::cpi::accounts::MintToken;
use mint_manager::{self};
use std::collections::BTreeMap;
use std::convert::Into;

pub use crate::errors::*;
pub use crate::states::*;
pub use crate::tx_accounts::*;

declare_id!("GF5hXVTvkErn2LTL5myFVbgqPXHnZYj2CkVGU6ZTEtyK");

#[program]
pub mod bridge_manager {
    use anchor_spl::token::{set_authority, SetAuthority};

    use super::*;
    // Initializes a new bridge account with a set of owners and a threshold.
    pub fn create_bridge(
        ctx: Context<CreateBridge>,
        owners: Vec<Pubkey>,
        threshold: u64,
        nonce: u8,
        support_chain_ids: Vec<u8>,
        resource_id_to_mint: BTreeMap<[u8; 32], Pubkey>,
        admin: Pubkey,
        fee_receiver: Pubkey,
        fee_amounts: BTreeMap<u8, u64>,
    ) -> Result<()> {
        msg!("stafi: create bridge");
        let bridge = &mut ctx.accounts.bridge;
        bridge.owners = owners;
        bridge.threshold = threshold;
        bridge.nonce = nonce;
        bridge.support_chain_ids = support_chain_ids;
        bridge.owner_set_seqno = 0;
        bridge.deposit_counts = BTreeMap::new();
        bridge.resource_id_to_mint = resource_id_to_mint;
        bridge.admin = admin;
        bridge.fee_receiver = fee_receiver;
        bridge.fee_amounts = fee_amounts;
        msg!("stafi: create bridge ok");
        Ok(())
    }

    pub fn set_resource_id(
        ctx: Context<AdminAuth>,
        resource_id: [u8; 32],
        mint: Pubkey,
    ) -> Result<()> {
        let bridge = &mut ctx.accounts.bridge;
        bridge.resource_id_to_mint.insert(resource_id, mint);
        Ok(())
    }

    pub fn set_support_chain_ids(ctx: Context<AdminAuth>, chain_ids: Vec<u8>) -> Result<()> {
        let bridge = &mut ctx.accounts.bridge;
        bridge.support_chain_ids = chain_ids;
        Ok(())
    }

    pub fn set_fee_receiver(ctx: Context<AdminAuth>, fee_receiver: Pubkey) -> Result<()> {
        let bridge = &mut ctx.accounts.bridge;
        bridge.fee_receiver = fee_receiver;
        Ok(())
    }

    pub fn set_fee_amount(ctx: Context<AdminAuth>, dest_chain_id: u8, amount: u64) -> Result<()> {
        let bridge = &mut ctx.accounts.bridge;
        bridge.fee_amounts.insert(dest_chain_id, amount);
        Ok(())
    }

    // Sets the owners field on the bridge.
    pub fn set_owners(ctx: Context<AdminAuth>, owners: Vec<Pubkey>) -> Result<()> {
        let owners_len = owners.len() as u64;
        if owners_len == 0 {
            return err!(Errors::InvalidOwnerLength);
        }

        let bridge = &mut ctx.accounts.bridge;
        if owners_len < bridge.threshold {
            bridge.threshold = owners_len;
        }

        bridge.owners = owners;
        bridge.owner_set_seqno += 1;
        Ok(())
    }

    // change_threshold.
    pub fn change_threshold(ctx: Context<AdminAuth>, threshold: u64) -> Result<()> {
        if threshold == 0 {
            return err!(Errors::InvalidThreshold);
        }
        if threshold > ctx.accounts.bridge.owners.len() as u64 {
            return err!(Errors::InvalidThreshold);
        }
        let bridge = &mut ctx.accounts.bridge;
        bridge.threshold = threshold;
        Ok(())
    }

    pub fn set_mint_authority(
        ctx: Context<SetMintAuthority>,
        new_mint_authority: Pubkey,
    ) -> Result<()> {
        set_authority(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                SetAuthority {
                    current_authority: ctx.accounts.bridge_signer.to_account_info(),
                    account_or_mint: ctx.accounts.mint.to_account_info(),
                },
                &[&[
                    &ctx.accounts.bridge.key().to_bytes(),
                    &[ctx.accounts.bridge.nonce],
                ]],
            ),
            AuthorityType::MintTokens,
            Some(new_mint_authority),
        )?;

        Ok(())
    }

    #[event]
    pub struct EventTransferOut {
        pub transfer: Pubkey,
        pub receiver: Vec<u8>,
        pub amount: u64,
        pub dest_chain_id: u8,
        pub resource_id: [u8; 32],
        pub deposit_nonce: u64,
    }

    // Initiates a transfer by creating a deposit account
    pub fn transfer_out(
        ctx: Context<TransferOut>,
        amount: u64,
        receiver: Vec<u8>,
        dest_chain_id: u8,
    ) -> Result<()> {
        msg!("stafi: transfer out");
        //check mint
        let mint_of_from = token::accessor::mint(&ctx.accounts.from.to_account_info())?;
        if mint_of_from != ctx.accounts.mint.key() {
            return err!(Errors::InvalidFromAccount);
        }

        //check resource id
        let mut resource_id_opt: Option<[u8; 32]> = None;
        for (id, mint) in ctx.accounts.bridge.resource_id_to_mint.iter() {
            if *mint == mint_of_from {
                resource_id_opt = Some(*id);
            }
        }
        let resource_id = if let Some(id) = resource_id_opt {
            id
        } else {
            return err!(Errors::NotSupportMintType);
        };

        //check dest chain id is support
        if !ctx
            .accounts
            .bridge
            .support_chain_ids
            .contains(&dest_chain_id)
        {
            return err!(Errors::NotSupportChainId);
        };

        //check fee receiver
        if *ctx.accounts.fee_receiver.key != ctx.accounts.bridge.fee_receiver {
            return err!(Errors::InvalidFeeReceiver);
        }
        let fee = if let Some(f) = ctx.accounts.bridge.fee_amounts.get(&dest_chain_id) {
            *f
        } else {
            0
        };
        if fee > 0 {
            // pay fee
            anchor_lang::solana_program::program::invoke(
                &anchor_lang::solana_program::system_instruction::transfer(
                    &ctx.accounts.authority.key(),
                    &ctx.accounts.fee_receiver.key(),
                    fee,
                ),
                &[
                    ctx.accounts.authority.to_account_info(),
                    ctx.accounts.fee_receiver.to_account_info(),
                    ctx.accounts.system_program.to_account_info(),
                ],
            )?;
        }

        //burn token of from account
        burn(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                Burn {
                    mint: ctx.accounts.mint.to_account_info(),
                    from: ctx.accounts.from.to_account_info(),
                    authority: ctx.accounts.authority.to_account_info(),
                },
            ),
            amount,
        )?;

        // update bridge deposit counts
        let bridge_account = &mut ctx.accounts.bridge;
        let deposit_count = bridge_account
            .deposit_counts
            .entry(dest_chain_id)
            .or_insert(0);
        *deposit_count += 1;

        // emit log data
        emit!(EventTransferOut {
            transfer: ctx.accounts.from.key(),
            receiver: receiver,
            amount: amount,
            dest_chain_id: dest_chain_id,
            resource_id: resource_id,
            deposit_nonce: *deposit_count,
        });
        msg!("stafi: transfer out ok");
        Ok(())
    }

    // Creates a new mint proposal account
    pub fn create_mint_proposal(
        ctx: Context<CreateMintProposal>,
        resource_id: [u8; 32],
        amount: u64,
        token_program: Pubkey,
    ) -> Result<()> {
        msg!("stafi: create mint proposal");
        let _ = ctx
            .accounts
            .bridge
            .owners
            .iter()
            .position(|a| a == ctx.accounts.proposer.key)
            .ok_or(Errors::InvalidOwner)?;

        let mut signers = Vec::new();
        signers.resize(ctx.accounts.bridge.owners.len(), false);

        let p = &mut ctx.accounts.proposal;
        let mint_op = ctx.accounts.bridge.resource_id_to_mint.get(&resource_id);
        let mint = if let Some(m) = mint_op {
            m
        } else {
            return err!(Errors::InvalidResourceId);
        };

        // check token account mint info
        let mint_info = token::accessor::mint(&ctx.accounts.to.to_account_info())?;
        if *mint != mint_info {
            return err!(Errors::InvalidMintAccount);
        }

        p.mint = *mint;
        p.to = ctx.accounts.to.key();
        p.amount = amount;
        p.token_program = token_program;
        p.signers = signers;
        p.bridge = *ctx.accounts.bridge.to_account_info().key;
        p.did_execute = false;
        p.owner_set_seqno = ctx.accounts.bridge.owner_set_seqno;
        msg!("stafi: create mint proposal ok");
        Ok(())
    }

    // Approve and Executes the given proposal if threshold owners have signed it.
    pub fn approve_mint_proposal(ctx: Context<Approve>) -> Result<()> {
        msg!("stafi: approve_mint_proposal");
        let owner_index = ctx
            .accounts
            .bridge
            .owners
            .iter()
            .position(|a| a == ctx.accounts.approver.key)
            .ok_or(Errors::InvalidOwner)?;

        ctx.accounts.proposal.signers[owner_index] = true;
        // Do we have enough signers.
        let sig_count = ctx
            .accounts
            .proposal
            .signers
            .iter()
            .filter(|&did_sign| *did_sign)
            .count() as u64;

        if sig_count < ctx.accounts.bridge.threshold {
            msg!("stafi: approve ok but not execute");
            return Ok(());
        }

        // Has this been executed already?
        if ctx.accounts.proposal.did_execute {
            msg!("stafi: proposal already executed err");
            return err!(Errors::AlreadyExecuted);
        }

        if ctx.accounts.proposal.mint != ctx.accounts.mint.key() {
            return err!(Errors::InvalidMintAccount);
        }
        if ctx.accounts.proposal.to != ctx.accounts.to.key() {
            return err!(Errors::InvalidToAccount);
        }

        // Execute the mint proposal signed by the bridge.
        let cpi_program = ctx.accounts.mint_manager_program.to_account_info();
        let cpi_accounts = MintToken {
            mint_manager: ctx.accounts.mint_manager.to_account_info(),
            rsol_mint: ctx.accounts.mint.to_account_info(),
            mint_to: ctx.accounts.to.to_account_info(),
            mint_authority: ctx.accounts.mint_authority.to_account_info(),
            ext_mint_authority: ctx.accounts.bridge_signer.to_account_info(),
            token_program: ctx.accounts.token_program.to_account_info(),
        };
        mint_manager::cpi::mint_token(
            CpiContext::new(cpi_program, cpi_accounts).with_signer(&[&[
                &ctx.accounts.bridge.key().to_bytes(),
                &[ctx.accounts.bridge.nonce],
            ]]),
            ctx.accounts.proposal.amount,
        )?;

        // Burn the mint proposal to ensure one time use.
        ctx.accounts.proposal.did_execute = true;
        msg!("stafi: approve and execute proposal ok");
        Ok(())
    }
}
