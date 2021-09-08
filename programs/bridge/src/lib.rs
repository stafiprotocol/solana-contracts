//! An example of a bridge to execute arbitrary Solana proposals.
//!
//! This program can be used to allow a bridge to govern anything a regular
//! Pubkey can govern. One can use the bridge as a BPF program upgrade
//! authority, a mint authority, etc.
//!
//! To use, one must first create a `Bridge` account, specifying two important
//! parameters:
//!
//! 1. Owners - the set of addresses that sign proposals for the bridge.
//! 2. Threshold - the number of signers required to execute a proposal.
//!
//! Once the `Bridge` account is created, one can create a `Proposal`
//! account, specifying the parameters for a normal solana proposal.
//!
//! To sign, owners should invoke the `approve` instruction.

use anchor_lang::prelude::*;
use anchor_lang::solana_program;
use anchor_lang::solana_program::instruction::Instruction;
use std::convert::Into;
use std::collections::BTreeMap;

#[program]
pub mod bridge {
    use super::*;

    // Initializes a new bridge account with a set of owners and a threshold.
    pub fn create_bridge(
        ctx: Context<CreateBridge>,
        owners: Vec<Pubkey>,
        threshold: u64,
        nonce: u8,
        resource_id_to_token_program: BTreeMap<[u8;32], Pubkey>,
    ) -> Result<()> {
        let bridge = &mut ctx.accounts.bridge;
        bridge.owners = owners;
        bridge.threshold = threshold;
        bridge.nonce = nonce;
        bridge.owner_set_seqno = 0;
        bridge.deposit_counts = BTreeMap::new();
        bridge.resource_id_to_token_program = resource_id_to_token_program;
        Ok(())
    }

    // Creates a new proposal account
    // which must be one of the owners of the bridge.
    pub fn create_proposal(
        ctx: Context<CreateProposal>,
        pids: Vec<Pubkey>,
        accs: Vec<Vec<ProposalAccount>>,
        datas: Vec<Vec<u8>>,
    ) -> Result<()> {
        if pids.len() != accs.len() || pids.len() != datas.len() {
            return Err(ErrorCode::ParamLength.into());
        }
        let _ = ctx
            .accounts
            .bridge
            .owners
            .iter()
            .position(|a| a == ctx.accounts.proposer.key)
            .ok_or(ErrorCode::InvalidOwner)?;

        let mut signers = Vec::new();
        signers.resize(ctx.accounts.bridge.owners.len(), false);

        let tx = &mut ctx.accounts.proposal;
        tx.program_ids = pids;
        tx.accounts = accs;
        tx.datas = datas;
        tx.signers = signers;
        tx.bridge = *ctx.accounts.bridge.to_account_info().key;
        tx.did_execute = false;
        tx.owner_set_seqno = ctx.accounts.bridge.owner_set_seqno;

        Ok(())
    }

    // Approve and Executes the given proposal if threshold owners have signed it.
    pub fn approve(ctx: Context<Approve>) -> Result<()> {
        let owner_index = ctx
            .accounts
            .bridge
            .owners
            .iter()
            .position(|a| a == ctx.accounts.owner.key)
            .ok_or(ErrorCode::InvalidOwner)?;

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
            return Ok(());
        }

        // Has this been executed already?
        if ctx.accounts.proposal.did_execute {
            return Err(ErrorCode::AlreadyExecuted.into());
        }

        // Execute the proposal signed by the bridge.
        let mut ixs: Vec<Instruction> = (&*ctx.accounts.proposal).into();
        for ix in ixs.iter_mut() {
            ix.accounts = ix
                .accounts
                .iter()
                .map(|acc| {
                    let mut acc = acc.clone();
                    if &acc.pubkey == ctx.accounts.bridge_signer.key {
                        acc.is_signer = true;
                    }
                    acc
                })
                .collect();
        }

        let seeds = &[
            ctx.accounts.bridge.to_account_info().key.as_ref(),
            &[ctx.accounts.bridge.nonce],
        ];
        let signer = &[&seeds[..]];
        let accounts = ctx.remaining_accounts;
        for ix in ixs.iter() {
            solana_program::program::invoke_signed(ix, &accounts, signer)?;
        }

        // Burn the proposal to ensure one time use.
        ctx.accounts.proposal.did_execute = true;

        Ok(())
    }

    // Sets the owners field on the bridge. The only way this can be invoked
    // is via a recursive call from execute_proposal -> set_owners.
    pub fn set_owners(ctx: Context<Auth>, owners: Vec<Pubkey>) -> Result<()> {
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

    // Changes the execution threshold of the bridge. The only way this can be
    // invoked is via a recursive call from execute_proposal ->
    // change_threshold.
    pub fn change_threshold(ctx: Context<Auth>, threshold: u64) -> Result<()> {
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

#[derive(Accounts)]
pub struct Auth<'info> {
    #[account(mut)]
    bridge: ProgramAccount<'info, Bridge>,
    #[account(
        signer, 
        seeds = [bridge.to_account_info().key.as_ref()],
        bump = bridge.nonce,
    )]
    bridge_signer: AccountInfo<'info>,
}

#[derive(Accounts)]
pub struct CreateBridge<'info> {
    #[account(zero)]
    bridge: ProgramAccount<'info, Bridge>,
    rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
pub struct CreateProposal<'info> {
    bridge: ProgramAccount<'info, Bridge>,
    #[account(zero)]
    proposal: ProgramAccount<'info, Proposal>,
    // One of the owners. Checked in the handler.
    #[account(signer)]
    proposer: AccountInfo<'info>,
    rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
pub struct Approve<'info> {
    #[account(constraint = bridge.owner_set_seqno == proposal.owner_set_seqno)]
    bridge: ProgramAccount<'info, Bridge>,
    #[account(
        seeds = [bridge.to_account_info().key.as_ref()],
        bump = bridge.nonce,
    )]
    bridge_signer: AccountInfo<'info>,
    #[account(mut, has_one = bridge)]
    proposal: ProgramAccount<'info, Proposal>,
    // One of the bridge owners. Checked in the handler.
    #[account(signer)]
    owner: AccountInfo<'info>,
}

#[account]
pub struct Bridge {
    pub owners: Vec<Pubkey>,
    pub threshold: u64,
    pub nonce: u8,
    pub owner_set_seqno: u32,
    // destinationChainID => number of deposits
    pub deposit_counts: BTreeMap<u8, u64>,
    pub resource_id_to_token_program: BTreeMap<[u8;32], Pubkey>,
}

#[account]
pub struct Proposal {
    // The bridge account this proposal belongs to.
    pub bridge: Pubkey,
    // Target program to execute against.
    pub program_ids: Vec<Pubkey>,
    // Accounts requried for the proposal.
    pub accounts: Vec<Vec<ProposalAccount>>,
    // Instruction datas for the proposal.
    pub datas: Vec<Vec<u8>>,
    // signers[index] is true if bridge.owners[index] signed the proposal.
    pub signers: Vec<bool>,
    // Boolean ensuring one time execution.
    pub did_execute: bool,
    // Owner set sequence number.
    pub owner_set_seqno: u32,
}

impl From<&Proposal> for Vec<Instruction> {
    fn from(tx: &Proposal) -> Vec<Instruction> {
        let mut instructions: Vec<Instruction> = Vec::new();
        for (i, _pid) in tx.program_ids.iter().enumerate() {
            instructions.push(Instruction {
                program_id: tx.program_ids[i],
                accounts: tx.accounts[i].iter().map(AccountMeta::from).collect(),
                data: tx.datas[i].clone(),
            })
        }
        instructions
    }
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct ProposalAccount {
    pub pubkey: Pubkey,
    pub is_signer: bool,
    pub is_writable: bool,
}

impl From<&ProposalAccount> for AccountMeta {
    fn from(account: &ProposalAccount) -> AccountMeta {
        match account.is_writable {
            false => AccountMeta::new_readonly(account.pubkey, account.is_signer),
            true => AccountMeta::new(account.pubkey, account.is_signer),
        }
    }
}

impl From<&AccountMeta> for ProposalAccount {
    fn from(account_meta: &AccountMeta) -> ProposalAccount {
        ProposalAccount {
            pubkey: account_meta.pubkey,
            is_signer: account_meta.is_signer,
            is_writable: account_meta.is_writable,
        }
    }
}

#[error]
pub enum ErrorCode {
    #[msg("The given owner is not part of this bridge.")]
    InvalidOwner,
    #[msg("The given owners is empty.")]
    InvalidOwnerLength,
    #[msg("Not enough owners signed this proposal.")]
    NotEnoughSigners,
    #[msg("Cannot delete a proposal that has been signed by an owner.")]
    ProposalAlreadySigned,
    #[msg("Overflow when adding.")]
    Overflow,
    #[msg("Cannot delete a proposal the owner did not create.")]
    UnableToDelete,
    #[msg("The given proposal has already been executed.")]
    AlreadyExecuted,
    #[msg("Threshold must be less than or equal to the number of owners.")]
    InvalidThreshold,
    #[msg("program id account data must have same length")]
    ParamLength,
}
