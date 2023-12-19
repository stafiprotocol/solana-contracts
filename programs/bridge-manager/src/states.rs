use anchor_lang::prelude::*;
use std::collections::BTreeMap;

#[account]
pub struct BridgeManagerAccount {
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
    pub fee_receiver: Pubkey,
    // destinationChainID => fee amount of sol
    pub fee_amounts: BTreeMap<u8, u64>,
}

#[account]
pub struct MintProposalAccount {
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
