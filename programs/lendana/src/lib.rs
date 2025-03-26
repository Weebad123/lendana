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

    // INITIALIZE THE GLOBAL REGISTRY OF WHITELISTED TOKENS AND POSITION COUNTERS
    pub fn init_whitelisted_registry_and_counters(ctx: Context<GlobalWhitelistedTokensAndPositionCounters>) -> Result<()> {

        instructions::admin_operations::init_global_tokens_and_counters(ctx)?;
        Ok(())
    }

    // WHITELIST A TOKEN BY THE WHITELISTER ROLE
    pub fn whitelist_token(ctx: Context<WhitelistToken>, token_mint: Pubkey) -> Result<()> {

        instructions::admin_operations::token_whitelist(ctx, token_mint)?;
        Ok(())
    }

    // LEND A TOKEN
    pub fn lend_token(ctx: Context<LenderPositionInfo>, amount_to_lend: u64, loan_terms: LoanTerms) -> Result<()> {

        instructions::lender_operations::create_lending_order(ctx, amount_to_lend, loan_terms)?;
        Ok(())
    }

    // MODIFY LENDER POSITION
    pub fn modify_lender_position(ctx: Context<ModifyLenderPosition>, new_loan_terms: LoanTerms, add_lending_amount: u64) -> Result<()> {

        instructions::lender_operations::modify_lender_position(ctx, new_loan_terms, add_lending_amount)?;
        Ok(())
    }

    // CANCEL LENDING ORDER
    pub fn cancel_lending_order(ctx: Context<CancelLendingOrder>) -> Result<()> {

        instructions::lender_operations::cancel_lending_order(ctx)?;
        Ok(())
    }
}

