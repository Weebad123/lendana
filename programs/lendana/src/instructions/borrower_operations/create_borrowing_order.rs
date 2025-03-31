

use anchor_lang::prelude::*;

use crate::{states::{accounts::*, contexts::*, errors::*}, utils::validate_loan_terms, LoanTerms, NATIVE_SOL_MINT_ADDRESS};


/* CREATE BORROWING ORDER
1. Need to ensure that the ltv is at least 80% */
pub fn create_borrowing_order(ctx: Context<BorrowerPositionInfo>, collateral_token: Pubkey, borrowing_token: Pubkey, borrowing_amount: u64, loan_terms: LoanTerms) 
-> Result<()> {

    // Non-Zero Amount To Borrow
    require!(borrowing_amount > 0, LendanaError::ZeroAmount);

    // Validate Loan Terms
    validate_loan_terms::validate_loan(loan_terms)?;

    // Lock Borrower's Collateral
    let collateral_amount = ctx.accounts.lock_borrower_collateral(borrowing_amount)?;

    // Transfer Borrow Tokens To Borrower
    ctx.accounts.transfer_tokens_to_borrower(borrowing_amount)?;

    // Get Borrower Position and Position Counter
    let borrower_position = &mut ctx.accounts.borrower_position;
    let borrower_postion_counter = &mut ctx.accounts.borrowers_position_id_counter;

    // Assign Position ID to Borrower
    let borrower_position_id = borrower_postion_counter.borrowers_current_position_id + 1;

    // Update The Counter
    borrower_postion_counter.borrowers_current_position_id += 1;

    // Create Borrower Position
    borrower_position.set_inner(BorrowerPosition {
        collateral_token: collateral_token,
        borrowing_token: borrowing_token,
        collateral_amount,
        borrower_pubkey: ctx.accounts.borrower.key(),
        borrowing_amount,
        borrower_position_id,
        borrowing_terms: loan_terms,
        is_position_active: true,
        is_matched: false,
        borrowing_start: Clock::get()?.unix_timestamp,
        borrower_position_bump: ctx.bumps.borrower_position,
    });

    // Update Borrowing Token Escrow
    let borrowing_token_escrow = &mut ctx.accounts.borrowing_token_escrow;
    
    borrowing_token_escrow.total_borrowed_tokens = borrowing_token_escrow.total_borrowed_tokens
        .checked_add(borrowing_amount)
        .ok_or(LendanaError::TokenAdditionOverflow)?;

    // Update Collateral Token Escrow: could be a SOL Collateral or Collateral Token
    if collateral_token == NATIVE_SOL_MINT_ADDRESS {
        let sol_collateral_vault = &mut ctx.accounts.sol_collateral_vault;
        sol_collateral_vault.vault_balance = sol_collateral_vault.vault_balance
            .checked_add(collateral_amount)
            .ok_or(LendanaError::TokenAdditionOverflow)?;
    } else {
        let collateral_token_escrow = &mut ctx.accounts.collateral_token_escrow;
        collateral_token_escrow.total_lent_tokens = collateral_token_escrow.total_lent_tokens
            .checked_add(collateral_amount)
            .ok_or(LendanaError::TokenAdditionOverflow)?;
    }
    
    Ok(())
}