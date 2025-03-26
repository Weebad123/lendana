use anchor_lang::prelude::*;


#[error_code]
pub enum LendanaError {
    #[msg("Only Callable By Admin")]
    OnlyAdmin,

    #[msg("Only Callable By Whitelister")]
    OnlyWhitelister,

    #[msg("Token Mint Does Not Match")]
    MismatchedTokenMint,

    #[msg("Token Is Not Whitelisted")]
    NotWhitelistedToken,

    #[msg("Specified Interest Rate Is Abnormal")]
    InvalidInterestRate,

    #[msg("Specified lending Duration Is Not Supported")]
    UnsupportedLendingDuration,

    #[msg("Lending Amount Cannot Be Zero")]
    ZeroAmount,

    #[msg("Token Addition To Vault Overflow")]
    TokenAdditionOverflow,

    #[msg("Cannot Modify An Already Matched Order")]
    OrderAlreadyMatched,

    #[msg("Only Owning Lender Can Call Function")]
    UnauthorizedLender,

    #[msg("Insufficient Lent Tokens In Vault")]
    InsufficientLentTokens,
}