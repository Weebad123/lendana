
use anchor_lang::prelude::*;

use crate::states::{contexts::*, errors::*};



pub fn cancel_borrowing_order(ctx: Context<CancelBorrowOrder>) -> Result<()> {

    // Ensure Order Is Not Matched
    require!(ctx.accounts.borrower_position.is_matched == false, LendanaError::OrderAlreadyMatched);

    // Refund Borrowed Tokens Back Into The Vault
    ctx.accounts.refund_tokens_to_borrow_vault()?;

    // Refund Borrower's Collateral
    ctx.accounts.unlock_borrower_collateral()?;

    // Update Collateral and Borrowing Token Escrows
    let collateral_token_escrow = &mut ctx.accounts.collateral_token_escrow;
    collateral_token_escrow.total_lent_tokens = collateral_token_escrow.total_lent_tokens
        .checked_sub(ctx.accounts.borrower_position.collateral_amount)
        .ok_or(LendanaError::InsufficientLentTokens)?;
    
    let borrowing_token_escrow = &mut ctx.accounts.borrowing_token_escrow;
    borrowing_token_escrow.total_borrowed_tokens = borrowing_token_escrow.total_borrowed_tokens
        .checked_sub(ctx.accounts.borrower_position.borrowing_amount)
        .ok_or(LendanaError::InsufficientBorrowedTokens)?;

    // Update Borrower Position
    let borrower_position = &mut ctx.accounts.borrower_position;
    borrower_position.borrowing_amount = 0;
    borrower_position.collateral_amount = 0;


    // Let's Close Borrower Position
    borrower_position.close(ctx.accounts.borrower.to_account_info())?;

    Ok(())
}