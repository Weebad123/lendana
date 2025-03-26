use anchor_lang::prelude::*;

use crate::states::{contexts::*, errors::*};


// @dev Should Only Be Allowed If There Is No Matching Yet
// @note Should Only Close The Lender Position PDA account, and refund the tokens to the lender

pub fn cancel_lending_order(ctx: Context<CancelLendingOrder>) -> Result<()> {

    //@dev Refund The Lender
    ctx.accounts.refund_tokens_to_lender()?;

    // Get Lender position for validation and checks
    let lender_position = &mut ctx.accounts.lender_position;
    let token_escrow_data = &mut ctx.accounts.token_escrow;

    // Update Token holdings of Lent Tokens
    token_escrow_data.total_lent_tokens = token_escrow_data.total_lent_tokens.checked_sub(lender_position.lending_amount).ok_or(LendanaError::InsufficientLentTokens)?;

    let lender_closing = ctx.accounts.lender.to_account_info();
    
    //@dev Close The Lender Position
    lender_position.close(lender_closing)?;

    Ok(())
}