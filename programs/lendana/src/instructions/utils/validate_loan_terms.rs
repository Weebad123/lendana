use anchor_lang::prelude::*;

use crate::states::{ accounts::*, errors::*, constants::*};

pub fn validate_loan(loan_terms: LoanTerms) -> Result<()> {
    /*  Ensure Lending Duration Meets 3 of the Available durations, 1, 3 or 6 Months
    require!(loan_terms.lending_duration == SIX_MONTH_LENDING_DURATION ||
            loan_terms.lending_duration == THREE_MONTH_LENDING_DURATION ||
            loan_terms.lending_duration == ONE_MONTH_LENDING_DURATION, 
            LendanaError::UnsupportedLendingDuration);*/
    /*If loan duration is more than half year, interest rate could be set to 7% max
    + However, if duration is 3 months, max interest rate is 5%
    + And if a month, max interest rate is 3%*/
    match loan_terms.lending_duration {
        SIX_MONTH_LENDING_DURATION => {
            require!(loan_terms.interest_rate <= 700, LendanaError::InvalidInterestRate);
        },
        THREE_MONTH_LENDING_DURATION => {
            require!(loan_terms.interest_rate <= 500, LendanaError::InvalidInterestRate);
        },
        ONE_MONTH_LENDING_DURATION => {
            require!(loan_terms.interest_rate <= 300, LendanaError::InvalidInterestRate);
        },
        _ => {
            return err!(LendanaError::UnsupportedLendingDuration);
        }
    }

    Ok(())
}