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

    #[msg("Lending Order Has Not Been Matched Yet")]
    OrderNotMatched,

    #[msg("Borrowing Same Token Not Allowed")]
    BorrowingSameToken,

    #[msg("Specified Borrowing Token Does Not Match Its Token Mint")]
    MismatchBorrowToken,

    #[msg("Specified Collateral Token Does Not Match Its Token Mint")]
    MismatchCollateralToken,

    #[msg("Token Price Feeds Can Only Be Updated By Authorized Role")]
    UnauthorizedPriceUpdater,

    #[msg("Token Already Has A Price Feed In Registry")]
    TokenPriceAlreadyExists,

    #[msg("Token Price Feed ID Not Found In Registry")]
    PriceFeedIDNotFound,

    #[msg("Getting Collateral Required For Borrowing Fails")]
    GetCollateralError,
}