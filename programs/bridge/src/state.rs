use anchor_lang::prelude::*;
use anchor_spl::token::Burn;
use std::collections::BTreeMap;
use std::convert::Into;

#[derive(Accounts)]
pub struct AdminAuth<'info> {
    #[account(mut)]
    pub bridge: ProgramAccount<'info, Bridge>,
    #[account(signer, constraint = &bridge.admin == admin.key)]
    pub admin: AccountInfo<'info>,
}

#[derive(Accounts)]
pub struct CreateBridge<'info> {
    #[account(zero)]
    pub bridge: ProgramAccount<'info, Bridge>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
pub struct TransferOut<'info> {
    #[account(mut)]
    pub bridge: ProgramAccount<'info, Bridge>,
    //token account's owner
    #[account(signer)]
    pub authority: AccountInfo<'info>,
    #[account(mut)]
    pub mint: AccountInfo<'info>,
    #[account(mut)]
    pub from: AccountInfo<'info>,
    pub token_program: AccountInfo<'info>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
pub struct CreateMintProposal<'info> {
    pub bridge: ProgramAccount<'info, Bridge>,
    #[account(zero)]
    pub proposal: ProgramAccount<'info, MintProposal>,
    // token account which has been initiated
    pub to: AccountInfo<'info>,
    // One of the owners. Checked in the handler.
    #[account(signer)]
    pub proposer: AccountInfo<'info>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
pub struct Approve<'info> {
    #[account(constraint = bridge.owner_set_seqno == proposal.owner_set_seqno)]
    pub bridge: ProgramAccount<'info, Bridge>,
    #[account(
        seeds = [bridge.to_account_info().key.as_ref()],
        bump = bridge.nonce,
    )]
    pub bridge_signer: AccountInfo<'info>,
    #[account(mut, has_one = bridge)]
    pub proposal: ProgramAccount<'info, MintProposal>,
    // One of the bridge owners. Checked in the handler.
    #[account(signer)]
    pub approver: AccountInfo<'info>,

    #[account(mut)]
    pub mint: AccountInfo<'info>,
    // token account which has been initiated
    #[account(mut)]
    pub to: AccountInfo<'info>,
    pub token_program: AccountInfo<'info>,
}

#[account]
pub struct Bridge {
    pub admin: Pubkey,
    pub owners: Vec<Pubkey>,
    pub threshold: u64,
    pub nonce: u8,
    pub owner_set_seqno: u32,
    pub support_chain_ids: Vec<u8>,
    // destinationChainID => number of deposits
    pub deposit_counts: BTreeMap<u8, u64>,
    // resource id => token mint address
    pub resource_id_to_mint: BTreeMap<[u8; 32], Pubkey>,
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

#[account]
pub struct MintProposal {
    // The bridge account this proposal belongs to.
    pub bridge: Pubkey,
    // signers[index] is true if bridge.owners[index] signed the proposal.
    pub signers: Vec<bool>,
    // Boolean ensuring one time execution.
    pub did_execute: bool,
    // Owner set sequence number.
    pub owner_set_seqno: u32,
    // spl mint account
    pub mint: Pubkey,
    // mint to account
    pub to: Pubkey,
    // mint account
    pub amount: u64,
    //spl token program
    pub token_program: Pubkey,
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

impl<'a, 'b, 'c, 'info> From<&mut TransferOut<'info>>
    for CpiContext<'a, 'b, 'c, 'info, Burn<'info>>
{
    fn from(accounts: &mut TransferOut<'info>) -> CpiContext<'a, 'b, 'c, 'info, Burn<'info>> {
        let cpi_accounts = Burn {
            mint: accounts.mint.clone(),
            to: accounts.from.clone(),
            authority: accounts.authority.clone(),
        };
        let cpi_program = accounts.token_program.clone();
        CpiContext::new(cpi_program, cpi_accounts)
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
    #[msg("chain id not support")]
    NotSupportChainId,
    #[msg("chain id exist")]
    ChainIdExist,
    #[msg("chain id not exist")]
    ChainIdNotExist,
    #[msg("resource id not found")]
    InvalidResourceId,
    #[msg("mint account not match proposal's mint")]
    InvalidMintAccount,
    #[msg("to account not match proposal's to")]
    InvalidToAccount,
    #[msg("from account invalid")]
    InvalidFromAccount,
    #[msg("not support mint type")]
    NotSupportMintType,
}
