use anchor_lang::prelude::*;

#[error_code]
pub enum Errors {
    #[msg("Program id not match")]
    ProgramIdNotMatch,

    #[msg("Remaining accounts not match")]
    RemainingAccountsNotMatch,

    #[msg("Admin not match")]
    AdminNotMatch,

    #[msg("Balancer not match")]
    BalancerNotMatch,

    #[msg("Initialize data not match")]
    InitializeDataMatch,

    #[msg("Fee recipient not match")]
    FeeRecipientNotMatch,

    #[msg("Delegation empty")]
    DelegationEmpty,

    #[msg("Stake amount too low")]
    StakeAmountTooLow,

    #[msg("Stake account not active")]
    StakeAccountNotActive,

    #[msg("Stake account active")]
    StakeAccountActive,

    #[msg("Stake account with lockup")]
    StakeAccountWithLockup,

    #[msg("Unstake recipient not match")]
    UnstakeRecipientNotMatch,

    #[msg("Validator not exist")]
    ValidatorNotExist,

    #[msg("Validator already exist")]
    ValidatorAlreadyExist,

    #[msg("Validator not match")]
    ValidatorNotMatch,

    #[msg("Stake account already exist")]
    StakeAccountAlreadyExist,

    #[msg("Split stake account already exist")]
    SplitStakeAccountAlreadyExist,

    #[msg("Stake account not exist")]
    StakeAccountNotExist,

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

    #[msg("Era is processing")]
    EraIsProcessing,

    #[msg("Era is processed")]
    EraIsProcessed,

    #[msg("Era no need bond")]
    EraNoNeedBond,

    #[msg("Era no need unbond")]
    EraNoNeedUnBond,

    #[msg("Era no need update active")]
    EraNoNeedUpdateActive,

    #[msg("Era no need update rate")]
    EraNoNeedUpdateRate,

    #[msg("Amount unmatch")]
    AmountUnmatch,

    #[msg("Invalid unstake account")]
    InvalidUnstakeAccount,

    #[msg("Unstake account not claimable")]
    UnstakeAccountNotClaimable,

    #[msg("Unstake account amount zero")]
    UnstakeAccountAmountZero,

    #[msg("Pool balance not enough")]
    PoolBalanceNotEnough,

    #[msg("Unstake amount is zero")]
    UnstakeAmountIsZero,

    #[msg("Validators not equal")]
    ValidatorsNotEqual,

    #[msg("Rate change over limit")]
    RateChangeOverLimit,

    #[msg("Mint account not match")]
    MintAccountNotMatch,

    #[msg("Mint to owner not match")]
    MintToOwnerNotMatch,

    #[msg("Stake accounts len over limit")]
    StakeAccountsLenOverLimit,
}
