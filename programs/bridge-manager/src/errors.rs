use anchor_lang::prelude::*;

#[error_code]
pub enum Errors {
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
    #[msg("Program id account data must have same length")]
    ParamLength,
    #[msg("Chain id not support")]
    NotSupportChainId,
    #[msg("Chain id exist")]
    ChainIdExist,
    #[msg("Chain id not exist")]
    ChainIdNotExist,
    #[msg("Resource id not found")]
    InvalidResourceId,
    #[msg("Mint account not match proposal's mint")]
    InvalidMintAccount,
    #[msg("To account not match proposal's to")]
    InvalidToAccount,
    #[msg("From account invalid")]
    InvalidFromAccount,
    #[msg("Invalid fee receiver")]
    InvalidFeeReceiver,
    #[msg("Not support mint type")]
    NotSupportMintType,
    #[msg("Admin not match")]
    AdminNotMatch,
    #[msg("Program id not match")]
    ProgramIdNotMatch,
}
