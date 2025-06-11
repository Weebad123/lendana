
use anchor_lang::prelude::*;

use crate::{states::{contexts::*, constants::*, errors::*}, validate_loan_terms, LoanTerms};




/*
@dev Should Only Be Allowed If There Is No Matching Yet
@note If Only Increasing Borrowing Amount, But Maintaining loan terms, new_loan_terms should be same as previous
@note If only changing loan terms, but not borrowing more tokens, additional_borrow_amount should be 0  
@note If Changing both loan terms and topping up lending amount, set new values according to standard
*/

pub fn modify_borrowing_order(ctx: Context<ModifyBorrowerPosition>, new_loan_terms: LoanTerms, additional_borrow_amount: u64) -> Result<()> {

    // Get Borrower Position For Validation And Checks
    let borrower_position = &ctx.accounts.borrower_position;

    let mut new_collateral_to_lock = 0;

    // First Ensure Borrowing Order Is Not Matched
    require!(borrower_position.is_matched == false, LendanaError::OrderAlreadyMatched);

    // If Modifying Loan Terms, We Validate The New Loan Terms. If Not, We Just Skip It Due To Previous Validation
    if borrower_position.borrowing_terms != new_loan_terms {
        validate_loan_terms::validate_loan(new_loan_terms)?;
    }

    // If Borrowing More Tokens, Lock Additional Collateral Based On New Borrowing Amount, transfer the new tokens to the Borrower And Update Borrower Position
    if additional_borrow_amount > 0 {
        // lock Additional Collateral
        new_collateral_to_lock = ctx.accounts.lock_borrower_collateral(additional_borrow_amount)?;

        // Transfer New Borrow Tokens To Borrower
        ctx.accounts.transfer_tokens_to_borrower(additional_borrow_amount)?;
    }

    // Update Borrower Position Here
    {
        let borrower_position = &mut ctx.accounts.borrower_position;
        borrower_position.borrowing_amount = borrower_position.borrowing_amount.checked_add(additional_borrow_amount)
            .ok_or(LendanaError::TokenAdditionOverflow)?;

        borrower_position.collateral_amount = borrower_position.collateral_amount.checked_add(new_collateral_to_lock)
            .ok_or(LendanaError::TokenAdditionOverflow)?;

        borrower_position.borrowing_terms = new_loan_terms;
    }

    // Update Borrowing And Collateral Token Escrows
    let borrowing_token_escrow = &mut ctx.accounts.borrowing_token_escrow;
    
    borrowing_token_escrow.total_borrowed_tokens = borrowing_token_escrow.total_borrowed_tokens
        .checked_add(additional_borrow_amount)
        .ok_or(LendanaError::TokenAdditionOverflow)?;

    // Update Collateral Token Escrow: could be a SOL Collateral or Collateral Token
    if ctx.accounts.token_collateral.key() == NATIVE_SOL_MINT_ADDRESS {
        let sol_collateral_vault = &mut ctx.accounts.sol_collateral_vault;
        sol_collateral_vault.vault_balance = sol_collateral_vault.vault_balance
            .checked_add(new_collateral_to_lock)
            .ok_or(LendanaError::TokenAdditionOverflow)?;
    } else {
        let collateral_token_escrow = &mut ctx.accounts.collateral_token_escrow;
        collateral_token_escrow.total_lent_tokens = collateral_token_escrow.total_lent_tokens
            .checked_add(new_collateral_to_lock)
            .ok_or(LendanaError::TokenAdditionOverflow)?;
    }
        
    
    Ok(())
}