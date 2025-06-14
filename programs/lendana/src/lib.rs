use anchor_lang::prelude::*;

pub mod instructions;
pub mod states;

pub use instructions::*;
pub use states::*;

declare_id!("9KGSxvbEghBVBN2GF1KWb64r9Z7tjnCWFW1eSFVCK9hu");

#[program]
pub mod lendana {
    use super::*;

    // INITIALIZE THE ADMIN ACCOUNT
    pub fn initialize_admin(ctx: Context<InitializeAdmin>, admin_address: Pubkey) -> Result<()> {
        
        instructions::admin_operations::admin_initialize(ctx, admin_address)?;
        Ok(())
    }


    // INITIALIZE THE GLOBAL TRUSTED ROLES
    pub fn initialize_trusted_authority(ctx: Context<InitializeTrustedRoles>) -> Result<()> {

        instructions::admin_operations::initialize_trusted_entities(ctx)?;
        Ok(())
    }

    // CREATE WHITELISTER ROLE
    pub fn grant_whitelister(ctx: Context<InitializeWhiteLister>, whitelister_address: Pubkey) -> Result<()> {

        instructions::admin_operations::initialize_whitelister(ctx, whitelister_address)?;
        Ok(())
    }

    // INITIALIZE THE GLOBAL REGISTRY OF WHITELISTED TOKENS AND THEIR PRICE FEEDS, AND POSITION COUNTERS
    pub fn init_registries_and_counters(ctx: Context<GlobalWhitelistedTokensPositionCountersAndPriceRegistry>) -> Result<()> {

        instructions::admin_operations::init_tokens_registry_prices_and_counters(ctx)?;
        Ok(())
    }

    // WHITELIST A TOKEN BY THE WHITELISTER ROLE
    pub fn whitelist_token(ctx: Context<WhitelistToken>, token_mint: Pubkey) -> Result<()> {

        instructions::admin_operations::token_whitelist(ctx, token_mint)?;
        Ok(())
    }

    // AUTHORIZED UPDATER ADDS A PRICE FEED FOR A WHITELISTED TOKEN
    pub fn add_price(ctx: Context<AddTokenPriceMapping>, token_mint: Pubkey, price_feed_id: String) -> Result<()> {

        instructions::admin_operations::add_token_prices(ctx, token_mint, price_feed_id)?;
        Ok(())
    }

    // LEND A TOKEN
    pub fn lend_token(ctx: Context<LenderPositionInfo>, amount_to_lend: u64, loan_terms: LoanTerms) -> Result<()> {

        instructions::lender_operations::create_lending_order(ctx, amount_to_lend, loan_terms)?;
        Ok(())
    }

    // MODIFY LENDER POSITION
    pub fn update_lender_position(ctx: Context<ModifyLenderPosition>, new_loan_terms: LoanTerms, add_lending_amount: u64) -> Result<()> {

        instructions::lender_operations::modify_lender_position(ctx, new_loan_terms, add_lending_amount)?;
        Ok(())
    }

    // CANCEL LENDING ORDER
    pub fn cancel_lend_order(ctx: Context<CancelLendingOrder>) -> Result<()> {

        instructions::lender_operations::cancel_lending_order(ctx)?;
        Ok(())
    }

    // BORROW A TOKEN
    pub fn borrow_token(ctx: Context<BorrowerPositionInfo>, collateral_token: Pubkey, borrowing_token: Pubkey, borrowing_amount: u64, loan_terms: LoanTerms) -> Result<()> {

        instructions::borrower_operations::create_borrowing_order(ctx, collateral_token, borrowing_token, borrowing_amount, loan_terms)?;
        Ok(())
    }

    // MODIFY BORROWER POSITION
    pub fn update_borrower_position(ctx: Context<ModifyBorrowerPosition>, new_borrowing_terms: LoanTerms, additional_borrow_amount: u64) -> Result<()> {

        instructions::borrower_operations::modify_borrowing_order(ctx, new_borrowing_terms, additional_borrow_amount)?;
        Ok(())
    }

    // CANCEL BORROW ORDER
    pub fn cancel_borrow_order(ctx: Context<CancelBorrowOrder>) -> Result<()> {

        instructions::borrower_operations::cancel_borrowing_order(ctx)?;
        Ok(())
    }
}

