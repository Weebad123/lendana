use anchor_lang::prelude::*;

use crate::{states::{contexts::*, errors::*, accounts::*}, validate_loan_terms, LoanTerms};

/*
1. Deposit lending tokens from lender into the token Vault, 
and update the data in the token Escrow
2. Update Lender position */

pub fn create_lending_order(ctx: Context<LenderPositionInfo>, amount_to_lend: u64,
     loan_terms: LoanTerms) -> Result<()> {

    // Non Zero Amount To Lend
    require!(amount_to_lend > 0, LendanaError::ZeroAmount);

    // Validate Loan Terms
    validate_loan_terms::validate_loan(loan_terms)?;

    // Make Lending Deposit Into Token Vault
    ctx.accounts.deposit_to_token_vault(amount_to_lend)?;

    // Update The Token Escrow Data
    let token_escrow_data = &mut ctx.accounts.token_escrow;
    
    token_escrow_data.total_lent_tokens.checked_add(amount_to_lend).ok_or(LendanaError::TokenAdditionOverflow)?;

    // Update Lender Position
    let lender_position = &mut ctx.accounts.lender_position;
    let lender_postion_counter = &mut ctx.accounts.lenders_position_id_counter;

    // Assign Position ID to lender
    let lender_position_id = lender_postion_counter.lenders_current_position_id + 1;

    // Update The Counter
    lender_postion_counter.lenders_current_position_id += 1;

    // Set Lender Position
    lender_position.set_inner(LenderPosition{
        lending_token: ctx.accounts.token_to_lend.key(),
        lender_pubkey: ctx.accounts.lender.key(),
        lending_amount: amount_to_lend,
        interest_accumulated: 0,
        lender_position_id: lender_position_id,
        lending_terms: loan_terms,
        is_position_active: true,
        lending_start: Clock::get()?.unix_timestamp,
        lender_position_bump: ctx.bumps.lender_position,
    });
    
    Ok(())
}