use anchor_lang::prelude::*;

#[error_code]
pub enum Errors {
    #[msg("Program id not match")]
    ProgramIdNotMatch,

    #[msg("Remaining accounts not match")]
    RemainingAccountsNotMatch,

    #[msg("Admin not match")]
    AdminNotMatch,

    #[msg("Delegation empty")]
    DelegationEmpty,

    #[msg("Stake amount too low")]
    StakeAmountTooLow,

    #[msg("Stake account not active")]
    StakeAccountNotActive,

    #[msg("Stake account with lockup")]
    StakeAccountWithLockup,

    #[msg("Unstake recipient not match")]
    UnstakeRecipientNotMatch,

    #[msg("Validator not exist")]
    ValidatorNotExist,

    #[msg("Stake account already exist")]
    StakeAccountAlreadyExist,

    #[msg("Rent not enough")]
    RentNotEnough,

    #[msg("Balance not enough")]
    BalanceNotEnough,

    #[msg("Calulation fail")]
    CalculationFail,

    #[msg("Authority not match")]
    AuthorityNotMatch,

    #[msg("Era is latest")]
    EraIsLatest,

    #[msg("Invalid unstake account")]
    InvalidUnstakeAccount,

    #[msg("Unstake account not claimable")]
    UnstakeAccountNotClaimable,

    #[msg("Unstake account amount zero")]
    UnstakeAccountAmountZero,

    #[msg("Pool balance not enough")]
    PoolBalanceNotEnough,
}
