use anchor_lang::prelude::*;

use crate::states::{contexts::*, errors::*};


pub fn init_global_tokens_and_counters(ctx: Context<GlobalWhitelistedTokensAndPositionCounters>) -> Result<()> {

    let all_tokens = &mut ctx.accounts.all_whitelisted_tokens;
    let lender_position_counter = &mut ctx.accounts.lenders_position_id_counter;
    let borrower_position_counter = &mut ctx.accounts.borrowers_position_id_counter;

    // Initializing Lender Position ID Counter
    lender_position_counter.lenders_current_position_id = 0;
    lender_position_counter.lender_position_id_bump = ctx.bumps.lenders_position_id_counter;

    // Initializing Borrower Position ID Counter
    borrower_position_counter.borrowers_current_position_id = 0;
    borrower_position_counter.borrower_position_id_bump = ctx.bumps.borrowers_position_id_counter;

    // Initializing Global Whitelisted Tokens
    all_tokens.tokens_whitelisted = Vec::new();
    all_tokens.tokens_whitelisted_bump = ctx.bumps.all_whitelisted_tokens;
    
    Ok(())
}