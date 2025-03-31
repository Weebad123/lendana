use anchor_lang::prelude::*;

use anchor_spl::{associated_token::AssociatedToken, 
    token_interface::{Mint, TokenAccount, TokenInterface, transfer_checked, TransferChecked}
};

use crate::states::{accounts::*, errors::*};



#[derive(Accounts)]
pub struct LenderPositionInfo<'info> {

    #[account(mut)]
    pub lender: Signer<'info>,

    #[account(
        constraint = all_whitelisted_tokens.tokens_whitelisted.contains(&token_to_lend.key()) @LendanaError::NotWhitelistedToken,
    )]
    pub token_to_lend: InterfaceAccount<'info, Mint>,

    #[account(
        mut,
        associated_token::mint = token_to_lend,
        associated_token::authority = lender,
    )]
    pub lender_ata: InterfaceAccount<'info, TokenAccount>,

    // Global Registry Whitelisted: Ensure lending token is whitelisted
    #[account(
        seeds = [b"all_whitelisted_tokens"],
        bump = all_whitelisted_tokens.tokens_whitelisted_bump,
    )]
    pub all_whitelisted_tokens: Account<'info, AllWhitelistedTokens>,

    // Token Escrow Account to track total lent and borrowed tokens
    #[account(
        mut,
        seeds = [b"token_escrow", token_to_lend.key().as_ref()],
        bump = token_escrow.token_vault_bump
    )]
    pub token_escrow: Account<'info, LentBorrowedTokenEscrow>,

    // The Associated Token Esrow Vault
    #[account(
        mut,
        associated_token::mint = token_to_lend,
        associated_token::authority = token_escrow,
    )]
    pub token_vault: InterfaceAccount<'info, TokenAccount>,

    // Lender Position ID Counter
    #[account(
        mut,
        seeds = [b"lenders_position_id_counter"],
        bump = lenders_position_id_counter.lender_position_id_bump,
    )]
    pub lenders_position_id_counter: Account<'info, LenderPositionIDCounter>,

    // Create Lender Position
    #[account(
        init,
        payer = lender,
        space = 8 + LenderPosition::INIT_SPACE,
        seeds = [b"lender_position", lender.key().as_ref(), token_to_lend.key().as_ref()],
        bump
    )]
    pub lender_position: Account<'info, LenderPosition>,

    pub system_program: Program<'info, System>,

    pub token_program: Interface<'info, TokenInterface>,

    pub associated_token_program: Program<'info, AssociatedToken>,
}

impl<'info> LenderPositionInfo<'info> {
    pub fn deposit_to_token_vault(&mut self, amount: u64) -> Result<()> {

        // Let's CPI into the token transfer
        let token_program =self.token_program.to_account_info();
        
        let cpi_accounts = TransferChecked {
            from: self.lender_ata.to_account_info(),
            to: self.token_vault.to_account_info(),
            mint: self.token_to_lend.to_account_info(),
            authority: self.lender.to_account_info(),
        };
        
        let cpi_program = CpiContext::new(token_program, cpi_accounts);

        transfer_checked(cpi_program, amount, self.token_to_lend.decimals)?;

        Ok(())
    }
}




// ---------- MODIFY LENDER POSITION EITHER VIA CHANGING LOAN TERMS OR TOPPING UP LENDING AMOUNT ----------
#[derive(Accounts)]
pub struct ModifyLenderPosition<'info> {

    #[account(
        mut,
        constraint = lender.key() == lender_position.lender_pubkey @LendanaError::UnauthorizedLender,
    )]
    pub lender: Signer<'info>,

    pub token_to_lend: InterfaceAccount<'info, Mint>,

    #[account(
        mut,
        associated_token::mint = token_to_lend,
        associated_token::authority = lender,
    )]
    pub lender_ata: InterfaceAccount<'info, TokenAccount>,


    // Token Escrow Account to track total lent tokens
    #[account(
        mut,
        seeds = [b"token_escrow", token_to_lend.key().as_ref()],
        bump = token_escrow.token_vault_bump
    )]
    pub token_escrow: Account<'info, LentBorrowedTokenEscrow>,

    // The Associated Token Esrow Vault
    #[account(
        mut,
        associated_token::mint = token_to_lend,
        associated_token::authority = token_escrow,
    )]
    pub token_vault: InterfaceAccount<'info, TokenAccount>,

    // Lender Position
    #[account(
        mut,
        seeds = [b"lender_position", lender.key().as_ref(), token_to_lend.key().as_ref()],
        bump = lender_position.lender_position_bump,
    )]
    pub lender_position: Account<'info, LenderPosition>,

    pub system_program: Program<'info, System>,

    pub token_program: Interface<'info, TokenInterface>,

    pub associated_token_program: Program<'info, AssociatedToken>,
}

impl<'info> ModifyLenderPosition<'info> {
    pub fn increase_lending_amount(&mut self, amount: u64) -> Result<()> {

        // Let's CPI into the token transfer
        let token_program =self.token_program.to_account_info();
        
        let cpi_accounts = TransferChecked {
            from: self.lender_ata.to_account_info(),
            to: self.token_vault.to_account_info(),
            mint: self.token_to_lend.to_account_info(),
            authority: self.lender.to_account_info(),
        };
        
        let cpi_program = CpiContext::new(token_program, cpi_accounts);

        transfer_checked(cpi_program, amount, self.token_to_lend.decimals)?;

        Ok(())
    }
}




// ---------- CANCEL LENDING ORDER ----------
#[derive(Accounts)]
pub struct CancelLendingOrder<'info> {

    #[account(
        mut,
        constraint = lender.key() == lender_position.lender_pubkey @LendanaError::UnauthorizedLender,
        constraint = lender_position.is_matched == false @LendanaError::OrderAlreadyMatched,
    )]
    pub lender: Signer<'info>,

    pub token_to_lend: InterfaceAccount<'info, Mint>,

    #[account(
        mut,
        associated_token::mint = token_to_lend,
        associated_token::authority = lender,
    )]
    pub lender_ata: InterfaceAccount<'info, TokenAccount>,

    #[account(
        mut,
        seeds = [b"token_escrow", token_to_lend.key().as_ref()],
        bump = token_escrow.token_vault_bump
    )]
    pub token_escrow: Account<'info, LentBorrowedTokenEscrow>,

    // The Associated Token Esrow Vault
    #[account(
        mut,
        associated_token::mint = token_to_lend,
        associated_token::authority = token_escrow,
    )]
    pub token_vault: InterfaceAccount<'info, TokenAccount>,

    // Lender Position
    #[account(
        mut,
        close = lender,
        seeds = [b"lender_position", lender.key().as_ref(), token_to_lend.key().as_ref()],
        bump = lender_position.lender_position_bump,
    )]
    pub lender_position: Account<'info, LenderPosition>,

    pub system_program: Program<'info, System>,

    pub token_program: Interface<'info, TokenInterface>,

    pub associated_token_program: Program<'info, AssociatedToken>,
}

impl<'info> CancelLendingOrder<'info> {
    pub fn refund_tokens_to_lender(&mut self) -> Result<()> {

        // Let's CPI into the token transfer
        let token_program =self.token_program.to_account_info();
        
        let cpi_accounts = TransferChecked {
            from: self.token_vault.to_account_info(),
            to: self.lender_ata.to_account_info(),
            mint: self.token_to_lend.to_account_info(),
            authority: self.token_escrow.to_account_info(),
        };

        let token_to_refund = self.token_to_lend.key();

        let seeds = &[
            b"token_escrow", 
            token_to_refund.as_ref(),
            &[self.token_escrow.token_vault_bump]
            ];

        let signer_seeds = &[&seeds[..]];
        
        let cpi_program = CpiContext::new_with_signer(token_program, cpi_accounts, signer_seeds);

        transfer_checked(cpi_program, self.lender_position.lending_amount, self.token_to_lend.decimals)?;

        Ok(())
    }
}


/* --------------------- WITHDRAW OF ACCUMULATED INTERESTS   --------------------- */
#[derive(Accounts)]
pub struct WithdrawLendingInterest<'info> {
    #[account(
        mut,
        constraint = lender.key() == lender_position.lender_pubkey @LendanaError::UnauthorizedLender,
        constraint = lender_position.is_matched == true @LendanaError::OrderNotMatched,
    )]
    pub lender: Signer<'info>,

    pub token_to_lend: InterfaceAccount<'info, Mint>,

    #[account(
        mut,
        associated_token::mint = token_to_lend,
        associated_token::authority = lender,
    )]
    pub lender_ata: InterfaceAccount<'info, TokenAccount>,

    #[account(
        mut,
        seeds = [b"token_escrow", token_to_lend.key().as_ref()],
        bump = token_escrow.token_vault_bump
    )]
    pub token_escrow: Account<'info, LentBorrowedTokenEscrow>,

    // The Associated Token Esrow Vault
    #[account(
        mut,
        associated_token::mint = token_to_lend,
        associated_token::authority = token_escrow,
    )]
    pub token_vault: InterfaceAccount<'info, TokenAccount>,

    // Lender Position
    #[account(
        mut,
        close = lender,
        seeds = [b"lender_position", lender.key().as_ref(), token_to_lend.key().as_ref()],
        bump = lender_position.lender_position_bump,
    )]
    pub lender_position: Account<'info, LenderPosition>,

    pub system_program: Program<'info, System>,

    pub token_program: Interface<'info, TokenInterface>,

    pub associated_token_program: Program<'info, AssociatedToken>,
}
