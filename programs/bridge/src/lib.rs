//! stafi solana bridge.
use anchor_lang::prelude::*;
use anchor_spl::token::{self, MintTo};
use std::collections::BTreeMap;
use std::convert::Into;

mod state;
pub use crate::state::*;

declare_id!("Cqpdbx8h2uVj4s3aKHCnhTdbgxx9eRSFg3UwPE7te9M6");

#[program]
pub mod bridge {
    use super::*;
    // Initializes a new bridge account with a set of owners and a threshold.
    pub fn create_bridge(
        ctx: Context<CreateBridge>,
        owners: Vec<Pubkey>,
        threshold: u64,
        nonce: u8,
        resource_id_to_mint: BTreeMap<[u8; 32], Pubkey>,
        admin: Pubkey,
    ) -> Result<()> {
        let bridge = &mut ctx.accounts.bridge;
        bridge.owners = owners;
        bridge.threshold = threshold;
        bridge.nonce = nonce;
        bridge.owner_set_seqno = 0;
        bridge.deposit_counts = BTreeMap::new();
        bridge.resource_id_to_mint = resource_id_to_mint;
        bridge.admin = admin;
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

    // Initiates a transfer by creating a deposit account
    pub fn burn_and_create_deposit(
        ctx: Context<CreateDeposit>,
        amount: u64,
        receiver: Vec<u8>,
        dest_chain_id: u8,
        resource_id: [u8; 32],
    ) -> Result<()> {
        //burn
        token::burn(ctx.accounts.into(), amount)?;

        let bridge_account = &mut ctx.accounts.bridge;
        let deposit_account = &mut ctx.accounts.deposit;
        let deposit_count = bridge_account
            .deposit_counts
            .entry(dest_chain_id)
            .or_insert(0);

        // update bridge deposit counts
        *deposit_count += 1;
        // update deposit
        deposit_account.amount = amount;
        deposit_account.receiver = receiver;
        deposit_account.dest_chain_id = dest_chain_id;
        deposit_account.resource_id = resource_id;
        deposit_account.deposit_nonce = *deposit_count;
        Ok(())
    }

    // Creates a new mint proposal account
    pub fn create_mint_proposal(
        ctx: Context<CreateMintProposal>,
        resource_id: [u8; 32],
        amount: u64,
        token_program: Pubkey,
    ) -> Result<()> {
        let _ = ctx
            .accounts
            .bridge
            .owners
            .iter()
            .position(|a| a == ctx.accounts.proposer.key)
            .ok_or(ErrorCode::InvalidOwner)?;

        let mut signers = Vec::new();
        signers.resize(ctx.accounts.bridge.owners.len(), false);

        let p = &mut ctx.accounts.proposal;
        let mint_op = ctx.accounts.bridge.resource_id_to_mint.get(&resource_id);
        let mint = if let Some(m) = mint_op {
            m
        } else {
            return Err(ErrorCode::InvalidResourceId.into());
        };

        // check token account mint info
        let mint_info = token::accessor::mint(&ctx.accounts.to)?;
        if *mint != mint_info {
            return Err(ErrorCode::InvalidMintAccount.into());
        }

        p.mint = *mint;
        p.to = *ctx.accounts.to.key;
        p.amount = amount;
        p.token_program = token_program;
        p.signers = signers;
        p.bridge = *ctx.accounts.bridge.to_account_info().key;
        p.did_execute = false;
        p.owner_set_seqno = ctx.accounts.bridge.owner_set_seqno;

        Ok(())
    }

    // Approve and Executes the given proposal if threshold owners have signed it.
    pub fn approve_mint_proposal(ctx: Context<Approve>) -> Result<()> {
        msg!("000");
        let owner_index = ctx
            .accounts
            .bridge
            .owners
            .iter()
            .position(|a| a == ctx.accounts.approver.key)
            .ok_or(ErrorCode::InvalidOwner)?;

        ctx.accounts.proposal.signers[owner_index] = true;
        msg!("111");
        // Do we have enough signers.
        let sig_count = ctx
            .accounts
            .proposal
            .signers
            .iter()
            .filter(|&did_sign| *did_sign)
            .count() as u64;

        msg!("222");
        if sig_count < ctx.accounts.bridge.threshold {
            return Ok(());
        }

        // Has this been executed already?
        if ctx.accounts.proposal.did_execute {
            return Err(ErrorCode::AlreadyExecuted.into());
        }

        if ctx.accounts.proposal.mint != *ctx.accounts.mint.key {
            return Err(ErrorCode::InvalidMintAccount.into());
        }
        if ctx.accounts.proposal.to != *ctx.accounts.to.key {
            return Err(ErrorCode::InvalidToAccount.into());
        }

        // Execute the mint proposal signed by the bridge.
        let amount = ctx.accounts.proposal.amount;
        let cpi_accounts = MintTo {
            mint: ctx.accounts.mint.clone(),
            to: ctx.accounts.to.clone(),
            authority: ctx.accounts.bridge_signer.clone(),
        };
        msg!("222");
        let cpi_program = ctx.accounts.token_program.clone();
        let seeds = &[
            ctx.accounts.bridge.to_account_info().key.as_ref(),
            &[ctx.accounts.bridge.nonce],
        ];
        let signer = &[&seeds[..]];
        let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer);
        msg!("333");
        token::mint_to(cpi_ctx, amount)?;
        msg!("444");
        // Burn the mint proposal to ensure one time use.
        ctx.accounts.proposal.did_execute = true;

        Ok(())
    }

    // Sets the owners field on the bridge. 
    pub fn set_owners(ctx: Context<AdminAuth>, owners: Vec<Pubkey>) -> Result<()> {
        let owners_len = owners.len() as u64;
        if owners_len == 0 {
            return Err(ErrorCode::InvalidOwnerLength.into());
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
            return Err(ErrorCode::InvalidThreshold.into());
        }
        if threshold > ctx.accounts.bridge.owners.len() as u64 {
            return Err(ErrorCode::InvalidThreshold.into());
        }
        let bridge = &mut ctx.accounts.bridge;
        bridge.threshold = threshold;
        Ok(())
    }
}
