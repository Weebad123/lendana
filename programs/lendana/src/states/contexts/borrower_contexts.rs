

use anchor_lang::{prelude::*, system_program::{Transfer, transfer}};

use anchor_spl::{associated_token::AssociatedToken, token_interface::{Mint, TokenAccount, TokenInterface, 
transfer_checked, TransferChecked}};

use crate::states::{accounts::*, errors::*, constants::*};

use pyth_solana_receiver_sdk::{ID as PYTH_PROGRAM_ID, price_update::{get_feed_id_from_hex, PriceUpdateV2}};




//** BORROWER POSITION INFO */

#[derive(Accounts)]
#[instruction(collateral_token: Pubkey, borrowing_token: Pubkey)]
pub struct BorrowerPositionInfo<'info> {
    #[account(mut)]
    pub borrower: Signer<'info>,

    #[account(
        constraint = all_whitelisted_tokens.tokens_whitelisted.contains(&token_to_borrow.key()) @LendanaError::NotWhitelistedToken,
        constraint = borrowing_token.key() == token_to_borrow.key() @LendanaError::MismatchBorrowToken,
    )]
    pub token_to_borrow: InterfaceAccount<'info, Mint>,

    #[account(
        constraint = (all_whitelisted_tokens.tokens_whitelisted.contains(&token_collateral.key()) ||
        token_collateral.key() == NATIVE_SOL_MINT_ADDRESS) @LendanaError::NotWhitelistedToken,
        constraint = token_collateral.key() != token_to_borrow.key() @LendanaError::BorrowingSameToken,
        constraint = token_collateral.key() == collateral_token @LendanaError::MismatchCollateralToken,
    )]
    pub token_collateral: InterfaceAccount<'info, Mint>,

    // Pyth Price Update oracle Account for the Collateral and Borrowing Token
    pub collateral_price_update: Account<'info, PriceUpdateV2>,

    pub borrowing_price_update: Account<'info, PriceUpdateV2>,

    // Borrower's Associated Token Accounts For Collateral Token and Borrowing Token
    #[account(
        mut,
        associated_token::mint = token_collateral,
        associated_token::authority = borrower,
    )]
    pub borrower_collateral_ata: Box<InterfaceAccount<'info, TokenAccount>>,

    // Init if borrower does not have an ATA for the borrowing token
    #[account(
        init_if_needed,
        payer = borrower,
        associated_token::mint = token_to_borrow,
        associated_token::authority = borrower,
    )]
    pub borrower_borrowing_ata: Box<InterfaceAccount<'info, TokenAccount>>,

    // Global Registry Whitelisted: Ensure borrowing and Collateral tokens are whitelisted
    #[account(
        seeds = [b"all_whitelisted_tokens"],
        bump = all_whitelisted_tokens.tokens_whitelisted_bump,
    )]
    pub all_whitelisted_tokens: Box<Account<'info, AllWhitelistedTokens>>,

    // Borrowing Token Escrow Account to track total lent and Borrowed tokens
    #[account(
        mut,
        seeds = [b"token_escrow", token_to_borrow.key().as_ref()],
        bump = borrowing_token_escrow.token_vault_bump
    )]
    pub borrowing_token_escrow: Box<Account<'info, LentBorrowedTokenEscrow>>,

    // Collateral Token Escrow Account to track total lent and Borrowed tokens
    #[account(
        mut,
        seeds = [b"token_escrow", token_collateral.key().as_ref()],
        bump = collateral_token_escrow.token_vault_bump
    )]
    pub collateral_token_escrow: Box<Account<'info, LentBorrowedTokenEscrow>>,

    // The Associated Borrowing Token Esrow Vault
    #[account(
        mut,
        associated_token::mint = token_to_borrow,
        associated_token::authority = borrowing_token_escrow,
    )]
    pub borrowing_token_vault: Box<InterfaceAccount<'info, TokenAccount>>,

     // The Associated Collateral Token Esrow Vault
     #[account(
        mut,
        associated_token::mint = token_collateral,
        associated_token::authority = collateral_token_escrow,
    )]
    pub collateral_token_vault: Box<InterfaceAccount<'info, TokenAccount>>,

    // The SOL Collateral Vault PDA
    #[account(
        mut,
        seeds = [b"sol_collateral_vault"],
        bump = sol_collateral_vault.vault_bump,
    )]
    pub sol_collateral_vault: Box<Account<'info, SolCollateralVault>>,

    // Borrower Position ID Counter
    #[account(
        mut,
        seeds = [b"borrowers_position_id_counter"],
        bump = borrowers_position_id_counter.borrower_position_id_bump,
    )]
    pub borrowers_position_id_counter: Box<Account<'info, BorrowerPositionIDCounter>>,

    // Create Borrower Position
    #[account(
        init,
        payer = borrower,
        space = 8 + BorrowerPosition::INIT_SPACE,
        seeds = [b"borrower_position", borrower.key().as_ref(), token_to_borrow.key().as_ref()],
        bump
    )]
    pub borrower_position: Box<Account<'info, BorrowerPosition>>,

    // Get Token Price Feed IDs From The Token Price Registry
    #[account(
        mut,
        seeds = [b"token_price_registry"],
        bump = tokens_price_feed_registry.registry_bump,
    )]
    pub tokens_price_feed_registry: Box<Account<'info, TokenPriceFeedRegistry>>,

    /// CHECK: This is safe as Pyth Program, no data will be read or write to
    #[account(
        address = PYTH_PROGRAM_ID
    )]
    pub pyth_program: UncheckedAccount<'info>,

    pub associated_token_program: Program<'info, AssociatedToken>,

    pub token_program: Interface<'info, TokenInterface>,

    pub system_program: Program<'info, System>,
}


impl<'info> BorrowerPositionInfo<'info> {

    // ASSOCIATED FUNCTION TO LOCK BORROWER'S Collateral
    pub fn lock_borrower_collateral(&mut self, borrowing_amount: u64) -> Result<u64> {
    
        // Get Required collateral
        let required_collateral_amount = self.calculate_collateral_needed(borrowing_amount)?;

        // Collateral Could Be native SOL or an SPL Token, so need to handle either case differently
        match self.token_collateral.key(){
            NATIVE_SOL_MINT_ADDRESS => {
                // Handle CPI transfers of SOL from borrower to SolCollateralVault pda
                let cpi_program = self.system_program.to_account_info();
                let cpi_accounts = Transfer {
                    from: self.borrower.to_account_info(),
                    to: self.sol_collateral_vault.to_account_info(),
                };
                let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
                transfer(cpi_ctx, required_collateral_amount)?;
            },
            _ => {
                // Handle CPI transfers of SPL Tokens
                let cpi_program = self.token_program.to_account_info();
                let cpi_accounts = TransferChecked {
                    from: self.borrower_collateral_ata.to_account_info(),
                    to: self.collateral_token_vault.to_account_info(),
                    mint: self.token_collateral.to_account_info(),
                    authority: self.borrower.to_account_info(),
                };

                let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
                transfer_checked(cpi_ctx, required_collateral_amount, self.token_collateral.decimals)?;
            }
        }

        Ok(required_collateral_amount)
    }


    // ASSOCIATED FUNCTION TO TRANSFER BORROW TOKENS TO BORROWER 
    pub fn transfer_tokens_to_borrower(&mut self, borrowing_amount: u64) -> Result<()> {
        // Transfer the borrowing amount to the borrower's ATA

        let borrowing_token = self.token_to_borrow.key();
        let cpi_program = self.token_program.to_account_info();
        let cpi_accounts = TransferChecked {
            from: self.borrowing_token_vault.to_account_info(),
            to: self.borrower_borrowing_ata.to_account_info(),
            mint: self.token_to_borrow.to_account_info(),
            authority: self.borrowing_token_escrow.to_account_info(),
        };

        let seeds = &[
            b"token_escrow",
            borrowing_token.as_ref(),
            &[self.borrowing_token_escrow.token_vault_bump],
        ];
        let signer_seeds = &[&seeds[..]];

        let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer_seeds);
        transfer_checked(cpi_ctx, borrowing_amount, self.token_to_borrow.decimals)?;

        Ok(())
    }

    // A method to calculate the required collateral amount to collateralize based on borrowing amount and LTV
    fn calculate_collateral_needed(&mut self, borrowing_amount: u64) -> Result<u64> {

        // Get the price of the collateral token in USD
        let collateral_token = &self.token_collateral.key();
        let collateral_price_feed_id_hex = match self.tokens_price_feed_registry.token_price_mapping.iter()
            .find(|m| &m.token_mint == collateral_token)
            .map(|m| m.price_feed_id.clone()){
                Some(feed_id) => feed_id,
                None => return Ok(0),
            };

        let collateral_feed_id = match get_feed_id_from_hex(&collateral_price_feed_id_hex) {
            Ok(feed_id) => feed_id,
            Err(_) => return Ok(0),
        };

        let collateral_price = self.collateral_price_update.get_price_no_older_than(
            &Clock::get()?, MAX_PRICE_FEED_AGE, &collateral_feed_id)?;
        let actual_collateral_price_value = collateral_price.price
            .checked_mul(10u64.pow(collateral_price.exponent.try_into().unwrap()).try_into().unwrap());

        // Get the price of the borrowing token in USD
        let borrowing_token = &self.token_to_borrow.key();
        let borrowing_price_feed_id_hex = match self.tokens_price_feed_registry.token_price_mapping.iter()
            .find(|m| &m.token_mint == borrowing_token)
            .map(|m| m.price_feed_id.clone()){
                Some(feed_id) => feed_id,
                None => return Ok(0),
            };
        let borrowing_feed_id = match get_feed_id_from_hex(&borrowing_price_feed_id_hex) {
            Ok(feed_id) => feed_id,
            Err(_) => return Ok(0),
        };
        
        let borrowing_token_price = self.borrowing_price_update.get_price_no_older_than(
            &Clock::get()?, MAX_PRICE_FEED_AGE, &borrowing_feed_id)?;
        let actual_borrowing_price_value = borrowing_token_price.price
            .checked_mul(10u64.pow(borrowing_token_price.exponent.try_into().unwrap()).try_into().unwrap());


        // Get Price of Borrowing Amount, and Calculate Required Collateral
        let borrowing_amount_price = borrowing_amount.checked_mul(actual_borrowing_price_value.unwrap().try_into().unwrap());

        /* 1. Min Collateral Ratio + Max Interest = Required Collateral */
        const EXPECTED_COLLATERAL_RATIO_AND_MAX_INTEREST_BPS: u64 = MAX_ALLOWABLE_INTEREST_RATE_BPS + MIN_COLLATERAL_RATIO; // 127% in BPS

        let required_collateral_value = borrowing_amount_price
            .unwrap()
            .checked_mul(EXPECTED_COLLATERAL_RATIO_AND_MAX_INTEREST_BPS)
            .unwrap()
            .checked_div(10000)
            .unwrap();
        
        let required_collateral_amount = required_collateral_value
            .checked_div(actual_collateral_price_value.unwrap().try_into().unwrap())
            .ok_or(LendanaError::GetCollateralError)?;


        Ok(required_collateral_amount)
    }
}



// Can Only Modify Position If There Is No Match Yet: Loan Terms Or Topping Up Amount
// MODIFY BORROWER POSITION   
#[derive(Accounts)]
pub struct ModifyBorrowerPosition<'info> {

    #[account(
        mut,
        constraint = borrower.key() == borrower_position.borrower_pubkey @LendanaError::UnauthorizedBorrower,
    )]
    pub borrower: Signer<'info>,

    #[account(
        constraint = token_to_borrow.key() == borrower_position.borrowing_token @LendanaError::MismatchBorrowToken,
    )]
    pub token_to_borrow: InterfaceAccount<'info, Mint>,

    #[account(
        constraint = token_collateral.key() == borrower_position.collateral_token @LendanaError::MismatchCollateralToken,
    )]
    pub token_collateral: InterfaceAccount<'info, Mint>,

     // Pyth Price Update oracle Account for the Collateral and Borrowing Token
     pub collateral_price_update: Account<'info, PriceUpdateV2>,

     pub borrowing_price_update: Account<'info, PriceUpdateV2>,
 
     // Borrower's Associated Token Accounts For Collateral Token and Borrowing Token
     #[account(
         mut,
         associated_token::mint = token_collateral,
         associated_token::authority = borrower,
     )]
     pub borrower_collateral_ata: Box<InterfaceAccount<'info, TokenAccount>>,
 
     // Init if borrower does not have an ATA for the borrowing token
     #[account(
        mut,
         associated_token::mint = token_to_borrow,
         associated_token::authority = borrower,
     )]
     pub borrower_borrowing_ata: Box<InterfaceAccount<'info, TokenAccount>>,

    // Borrowing Token Escrow Account to track total lent and Borrowed tokens
    #[account(
        mut,
        seeds = [b"token_escrow", token_to_borrow.key().as_ref()],
        bump = borrowing_token_escrow.token_vault_bump
    )]
    pub borrowing_token_escrow: Box<Account<'info, LentBorrowedTokenEscrow>>,

    // Collateral Token Escrow Account to track total lent and Borrowed tokens
    #[account(
        mut,
        seeds = [b"token_escrow", token_collateral.key().as_ref()],
        bump = collateral_token_escrow.token_vault_bump
    )]
    pub collateral_token_escrow: Box<Account<'info, LentBorrowedTokenEscrow>>,

    #[account(
        mut,
        associated_token::mint = token_to_borrow,
        associated_token::authority = borrowing_token_escrow,
    )]
    pub borrowing_token_vault: Box<InterfaceAccount<'info, TokenAccount>>,

     // The Associated Collateral Token Esrow Vault
     #[account(
        mut,
        associated_token::mint = token_collateral,
        associated_token::authority = collateral_token_escrow,
    )]
    pub collateral_token_vault: Box<InterfaceAccount<'info, TokenAccount>>,

    // The SOL Collateral Vault PDA
    #[account(
        mut,
        seeds = [b"sol_collateral_vault"],
        bump = sol_collateral_vault.vault_bump,
    )]
    pub sol_collateral_vault: Box<Account<'info, SolCollateralVault>>,

    // Borrower Position ID Counter
    #[account(
        //mut,
        seeds = [b"borrowers_position_id_counter"],
        bump = borrowers_position_id_counter.borrower_position_id_bump,
    )]
    pub borrowers_position_id_counter: Box<Account<'info, BorrowerPositionIDCounter>>,

    // Get Borrower Position
    #[account(
        mut,
        seeds = [b"borrower_position", borrower.key().as_ref(), token_to_borrow.key().as_ref()],
        bump = borrower_position.borrower_position_bump
    )]
    pub borrower_position: Box<Account<'info, BorrowerPosition>>,

     // Get Token Price Feed IDs From The Token Price Registry
    #[account(
        mut,
        seeds = [b"token_price_registry"],
        bump = tokens_price_feed_registry.registry_bump,
    )]
    pub tokens_price_feed_registry: Box<Account<'info, TokenPriceFeedRegistry>>,

    /// CHECK: This is safe as Pyth Program, no data will be read or write to
    #[account(
        address = PYTH_PROGRAM_ID
    )]
    pub pyth_program: UncheckedAccount<'info>,

    pub associated_token_program: Program<'info, AssociatedToken>,

    pub token_program: Interface<'info, TokenInterface>,

    pub system_program: Program<'info, System>,
}

 //@dev    todo! Modularize the usage of the same implemented functions via trait Usage or macro_rules on both ModifyBorrowerPosition and BorrowerPosition

impl<'info> ModifyBorrowerPosition<'info> {

     // ASSOCIATED FUNCTION TO LOCK BORROWER'S Collateral
     pub fn lock_borrower_collateral(&mut self, borrowing_amount: u64) -> Result<u64> {
    
        // Get Required collateral
        let required_collateral_amount = self.calculate_collateral_needed(borrowing_amount)?;

        // Collateral Could Be native SOL or an SPL Token, so need to handle either case differently
        match self.token_collateral.key(){
            NATIVE_SOL_MINT_ADDRESS => {
                // Handle CPI transfers of SOL from borrower to SolCollateralVault pda
                let cpi_program = self.system_program.to_account_info();
                let cpi_accounts = Transfer {
                    from: self.borrower.to_account_info(),
                    to: self.sol_collateral_vault.to_account_info(),
                };
                let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
                transfer(cpi_ctx, required_collateral_amount)?;
            },
            _ => {
                // Handle CPI transfers of SPL Tokens
                let cpi_program = self.token_program.to_account_info();
                let cpi_accounts = TransferChecked {
                    from: self.borrower_collateral_ata.to_account_info(),
                    to: self.collateral_token_vault.to_account_info(),
                    mint: self.token_collateral.to_account_info(),
                    authority: self.borrower.to_account_info(),
                };

                let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
                transfer_checked(cpi_ctx, required_collateral_amount, self.token_collateral.decimals)?;
            }
        }

        Ok(required_collateral_amount)
    }


    // ASSOCIATED FUNCTION TO TRANSFER BORROW TOKENS TO BORROWER 
    pub fn transfer_tokens_to_borrower(&mut self, borrowing_amount: u64) -> Result<()> {
        // Transfer the borrowing amount to the borrower's ATA

        let borrowing_token = self.token_to_borrow.key();
        let cpi_program = self.token_program.to_account_info();
        let cpi_accounts = TransferChecked {
            from: self.borrowing_token_vault.to_account_info(),
            to: self.borrower_borrowing_ata.to_account_info(),
            mint: self.token_to_borrow.to_account_info(),
            authority: self.borrowing_token_escrow.to_account_info(),
        };

        let seeds = &[
            b"token_escrow",
            borrowing_token.as_ref(),
            &[self.borrowing_token_escrow.token_vault_bump],
        ];
        let signer_seeds = &[&seeds[..]];

        let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer_seeds);
        transfer_checked(cpi_ctx, borrowing_amount, self.token_to_borrow.decimals)?;

        Ok(())
    }

    // A method to calculate the required collateral amount to collateralize based on borrowing amount and LTV
    fn calculate_collateral_needed(&mut self, borrowing_amount: u64) -> Result<u64> {

        // Get the price of the collateral token in USD
        let collateral_token = &self.token_collateral.key();
        let collateral_price_feed_id_hex = match self.tokens_price_feed_registry.token_price_mapping.iter()
            .find(|m| &m.token_mint == collateral_token)
            .map(|m| m.price_feed_id.clone()){
                Some(feed_id) => feed_id,
                None => return Ok(0),
            };

        let collateral_feed_id = match get_feed_id_from_hex(&collateral_price_feed_id_hex) {
            Ok(feed_id) => feed_id,
            Err(_) => return Ok(0),
        };

        let collateral_price = self.collateral_price_update.get_price_no_older_than(
            &Clock::get()?, MAX_PRICE_FEED_AGE, &collateral_feed_id)?;
        let actual_collateral_price_value = collateral_price.price
            .checked_mul(10u64.pow(collateral_price.exponent.try_into().unwrap()).try_into().unwrap());

        // Get the price of the borrowing token in USD
        let borrowing_token = &self.token_to_borrow.key();
        let borrowing_price_feed_id_hex = match self.tokens_price_feed_registry.token_price_mapping.iter()
            .find(|m| &m.token_mint == borrowing_token)
            .map(|m| m.price_feed_id.clone()){
                Some(feed_id) => feed_id,
                None => return Ok(0),
            };
        let borrowing_feed_id = match get_feed_id_from_hex(&borrowing_price_feed_id_hex) {
            Ok(feed_id) => feed_id,
            Err(_) => return Ok(0),
        };
        
        let borrowing_token_price = self.borrowing_price_update.get_price_no_older_than(
            &Clock::get()?, MAX_PRICE_FEED_AGE, &borrowing_feed_id)?;
        let actual_borrowing_price_value = borrowing_token_price.price
            .checked_mul(10u64.pow(borrowing_token_price.exponent.try_into().unwrap()).try_into().unwrap());


        // Get Price of Borrowing Amount, and Calculate Required Collateral
        let borrowing_amount_price = borrowing_amount.checked_mul(actual_borrowing_price_value.unwrap().try_into().unwrap());

        /* 1. Min Collateral Ratio + Max Interest = Required Collateral */
        const EXPECTED_COLLATERAL_RATIO_AND_MAX_INTEREST_BPS: u64 = MAX_ALLOWABLE_INTEREST_RATE_BPS + MIN_COLLATERAL_RATIO; // 127% in BPS

        let required_collateral_value = borrowing_amount_price
            .unwrap()
            .checked_mul(EXPECTED_COLLATERAL_RATIO_AND_MAX_INTEREST_BPS)
            .unwrap()
            .checked_div(10000)
            .unwrap();
        
        let required_collateral_amount = required_collateral_value
            .checked_div(actual_collateral_price_value.unwrap().try_into().unwrap())
            .ok_or(LendanaError::GetCollateralError)?;


        Ok(required_collateral_amount)
    }
}



// CANCEL BORROWING ORDER
#[derive(Accounts)]
pub struct CancelBorrowOrder<'info> {

    #[account(
        mut,
        constraint = borrower.key() == borrower_position.borrower_pubkey @LendanaError::UnauthorizedBorrower,
    )]
    pub borrower: Signer<'info>,

    #[account(
        constraint = token_to_borrow.key() == borrower_position.borrowing_token @LendanaError::MismatchBorrowToken,
    )]
    pub token_to_borrow: InterfaceAccount<'info, Mint>,

    #[account(
        constraint = token_collateral.key() == borrower_position.collateral_token @LendanaError::MismatchCollateralToken,
    )]
    pub token_collateral: InterfaceAccount<'info, Mint>,

     // Pyth Price Update oracle Account for the Collateral and Borrowing Token
     pub collateral_price_update: Account<'info, PriceUpdateV2>,

     pub borrowing_price_update: Account<'info, PriceUpdateV2>,
 
     // Borrower's Associated Token Accounts For Collateral Token and Borrowing Token
     #[account(
         mut,
         associated_token::mint = token_collateral,
         associated_token::authority = borrower,
     )]
     pub borrower_collateral_ata: Box<InterfaceAccount<'info, TokenAccount>>,
 
     // Init if borrower does not have an ATA for the borrowing token
     #[account(
        mut,
         associated_token::mint = token_to_borrow,
         associated_token::authority = borrower,
     )]
     pub borrower_borrowing_ata: Box<InterfaceAccount<'info, TokenAccount>>,

    // Borrowing Token Escrow Account to track total lent and Borrowed tokens
    #[account(
        mut,
        seeds = [b"token_escrow", token_to_borrow.key().as_ref()],
        bump = borrowing_token_escrow.token_vault_bump
    )]
    pub borrowing_token_escrow: Box<Account<'info, LentBorrowedTokenEscrow>>,

    // Collateral Token Escrow Account to track total lent and Borrowed tokens
    #[account(
        mut,
        seeds = [b"token_escrow", token_collateral.key().as_ref()],
        bump = collateral_token_escrow.token_vault_bump
    )]
    pub collateral_token_escrow: Box<Account<'info, LentBorrowedTokenEscrow>>,

    #[account(
        mut,
        associated_token::mint = token_to_borrow,
        associated_token::authority = borrowing_token_escrow,
    )]
    pub borrowing_token_vault: Box<InterfaceAccount<'info, TokenAccount>>,

     // The Associated Collateral Token Esrow Vault
     #[account(
        mut,
        associated_token::mint = token_collateral,
        associated_token::authority = collateral_token_escrow,
    )]
    pub collateral_token_vault: Box<InterfaceAccount<'info, TokenAccount>>,

    // The SOL Collateral Vault PDA
    #[account(
        mut,
        seeds = [b"sol_collateral_vault"],
        bump = sol_collateral_vault.vault_bump,
    )]
    pub sol_collateral_vault: Box<Account<'info, SolCollateralVault>>,

    // Borrower Position ID Counter
    #[account(
        //mut,
        seeds = [b"borrowers_position_id_counter"],
        bump = borrowers_position_id_counter.borrower_position_id_bump,
    )]
    pub borrowers_position_id_counter: Box<Account<'info, BorrowerPositionIDCounter>>,

    // Get Borrower Position
    #[account(
        mut,
        close = borrower,
        seeds = [b"borrower_position", borrower.key().as_ref(), token_to_borrow.key().as_ref()],
        bump = borrower_position.borrower_position_bump
    )]
    pub borrower_position: Box<Account<'info, BorrowerPosition>>,

     // Get Token Price Feed IDs From The Token Price Registry
    #[account(
        mut,
        seeds = [b"token_price_registry"],
        bump = tokens_price_feed_registry.registry_bump,
    )]
    pub tokens_price_feed_registry: Box<Account<'info, TokenPriceFeedRegistry>>,

    /// CHECK: This is safe as Pyth Program, no data will be read or write to
    #[account(
        address = PYTH_PROGRAM_ID
    )]
    pub pyth_program: UncheckedAccount<'info>,

    pub associated_token_program: Program<'info, AssociatedToken>,

    pub token_program: Interface<'info, TokenInterface>,

    pub system_program: Program<'info, System>,
}

impl<'info> CancelBorrowOrder<'info> {

    // Method 1: Refund Borrow Tokens
    pub fn refund_tokens_to_borrow_vault(&mut self) -> Result<()> {
        // Transfer the borrowed Tokens to the Borrow Vault ATA

        
        let borrowing_amount = self.borrower_position.borrowing_amount;
        let cpi_program = self.token_program.to_account_info();
        let cpi_accounts = TransferChecked {
            from: self.borrower_borrowing_ata.to_account_info(),
            to: self.borrowing_token_vault.to_account_info(),
            mint: self.token_to_borrow.to_account_info(),
            authority: self.borrower.to_account_info(),
        };

        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
        transfer_checked(cpi_ctx, borrowing_amount, self.token_to_borrow.decimals)?;

        Ok(())
    }


    // Method 2: Unlock And Send Borrower's Collateral
    pub fn unlock_borrower_collateral(&mut self) -> Result<()> {

        // Collateral Could Be Native SOL or One of The Whitelisted Tokens
        let collateral_to_unlock = self.borrower_position.collateral_amount;

        match self.token_collateral.key(){
            NATIVE_SOL_MINT_ADDRESS => {
                let cpi_program = self.system_program.to_account_info();
                let cpi_accounts = Transfer {
                    from: self.sol_collateral_vault.to_account_info(),
                    to: self.borrower.to_account_info()
                };

                let seeds = &[
                    b"sol_collateral_vault".as_ref(),
                    &[self.sol_collateral_vault.vault_bump],
                ];
                let signer_seeds = &[&seeds[..]];

                let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer_seeds);
                
                transfer(cpi_ctx, collateral_to_unlock)?;
            },
            // IF COLLATERAL TOKEN IS ANY SPL TOKEN
            _ => {
                let collateral_token = self.token_collateral.key();
                let cpi_program = self.token_program.to_account_info();
                let cpi_accounts = TransferChecked {
                    from: self.collateral_token_vault.to_account_info(),
                    mint: self.token_collateral.to_account_info(),
                    to: self.borrower_collateral_ata.to_account_info(),
                    authority: self.collateral_token_escrow.to_account_info()
                };

                let seeds = &[
                    b"token_escrow",
                    collateral_token.as_ref(),
                    &[self.collateral_token_escrow.token_vault_bump]
                ];
                let signer_seeds = &[&seeds[..]];

                let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer_seeds);

                transfer_checked(cpi_ctx, collateral_to_unlock, self.token_collateral.decimals)?;
            }
        }
        Ok(())
    }
 
}