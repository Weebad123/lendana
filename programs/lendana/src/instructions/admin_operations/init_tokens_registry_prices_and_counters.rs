use anchor_lang::prelude::*;

use crate::states::contexts::*;


pub fn init_tokens_registry_prices_and_counters(ctx: Context<GlobalWhitelistedTokensPositionCountersAndPriceRegistry>) -> Result<()> {

    let all_tokens = &mut ctx.accounts.all_whitelisted_tokens;
    let lender_position_counter = &mut ctx.accounts.lenders_position_id_counter;
    let borrower_position_counter = &mut ctx.accounts.borrowers_position_id_counter;
    let tokens_price_feed_registry = &mut ctx.accounts.tokens_price_feed_registry;
    let sol_collateral_vault = &mut ctx.accounts.sol_collateral_vault;

    // Initializing Lender Position ID Counter
    lender_position_counter.lenders_current_position_id = 0;
    lender_position_counter.lender_position_id_bump = ctx.bumps.lenders_position_id_counter;

    // Initializing Borrower Position ID Counter
    borrower_position_counter.borrowers_current_position_id = 0;
    borrower_position_counter.borrower_position_id_bump = ctx.bumps.borrowers_position_id_counter;

    // Initializing Global Whitelisted Tokens
    all_tokens.tokens_whitelisted = Vec::new();
    all_tokens.tokens_whitelisted_bump = ctx.bumps.all_whitelisted_tokens;

    // Initializing Token Price Feed Registry
    tokens_price_feed_registry.authority = ctx.accounts.whitelister_role.key();
    tokens_price_feed_registry.registry_bump = ctx.bumps.tokens_price_feed_registry;
    tokens_price_feed_registry.token_price_mapping = Vec::new();

    // Intializing The SOL Collateral PDA Vault
    sol_collateral_vault.vault_authority = ctx.accounts.whitelister_role.key();// !@note will change authority to some AdminConfig
    sol_collateral_vault.vault_bump = ctx.bumps.sol_collateral_vault;
    sol_collateral_vault.vault_balance = 0;
    sol_collateral_vault.is_active = true;
    
    Ok(())
}