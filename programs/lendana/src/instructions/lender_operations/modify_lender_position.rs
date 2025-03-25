use anchor_lang::prelude::*;

use crate::{states::{contexts::*, errors::*}, validate_loan_terms, LoanTerms};


/*
@dev Should Only Be Allowed If There Is No Matching Yet
@note If Only Increasing Lending Amount, But Maintaining loan terms, new_loan_terms should be same as previous
@note If only changing loan terms, but not topping up lending amount, add_lending_amount should be 0  
@note If Changing both loan terms and topping up lending amount, set new values according to standard
*/

pub fn modify_lender_position(ctx: Context<ModifyLenderPosition>, new_loan_terms: LoanTerms, add_lending_amount: u64) -> Result<()> {

    // Get Lender position for validation and checks
    let lender_position = &ctx.accounts.lender_position;
    
    // First Ensure Lending Order Is Not Matched
    require!(lender_position.is_matched == false, LendanaError::OrderAlreadyMatched);

    // If Modifying Loan Terms, Then We Validated The New Loan Terms. If Not, We Just Skip because of previous validation
    if lender_position.lending_terms != new_loan_terms {
        validate_loan_terms::validate_loan(new_loan_terms)?;
    }

    // If Topping Up Lending Amount, Retrieve Tokens From Lender ATA
    if add_lending_amount > 0 {
        ctx.accounts.increase_lending_amount(add_lending_amount)?;
        
        // Update The Token Escrow Data
        let token_escrow_data = &mut ctx.accounts.token_escrow;

        token_escrow_data.total_lent_tokens = token_escrow_data.total_lent_tokens
            .checked_add(add_lending_amount)
            .ok_or(LendanaError::TokenAdditionOverflow)?;
    }

    // Update Lender Position
    {
    let lender_position = &mut ctx.accounts.lender_position;
    lender_position.lending_amount = lender_position.lending_amount.checked_add(add_lending_amount).ok_or(LendanaError::TokenAdditionOverflow)?;
    lender_position.lending_terms = new_loan_terms;
    }


    Ok(())
}