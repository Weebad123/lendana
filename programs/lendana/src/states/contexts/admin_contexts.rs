use anchor_lang::prelude::*;

use anchor_spl::{associated_token::AssociatedToken, token_interface::{Mint, TokenAccount, TokenInterface}};

use crate::states::{accounts::*, errors::*};

/*
The Admin Context Struct
 */

 #[derive(Accounts)]
 #[instruction(admin_address: Pubkey)]
 pub struct InitializeAdmin<'info> {
    #[account(mut)]
    pub deployer: Signer<'info>,

    #[account(
        init,
        payer = deployer,
        seeds = [b"admin", admin_address.key().as_ref()],
        bump,
        space = 8 + 32 + 1,
    )]
    pub admin_account: Account<'info, Administrator>,

    pub system_program: Program<'info, System>,
 }

 /*
 The Trusted Entities Context Struct */
 #[derive(Accounts)]
 pub struct InitializeTrustedRoles<'info> {

    #[account(
        mut,
        constraint = admin.key() == admin_account.admin_address.key() @ LendanaError::OnlyAdmin
    )]
    pub admin: Signer<'info>,

    #[account(
        seeds = [b"admin", admin_account.admin_address.key().as_ref()],
        bump = admin_account.admin_bump
    )]
    pub admin_account: Account<'info, Administrator>,

    #[account(
        init,
        payer = admin,
        seeds = [b"trusted_entities"],
        bump,
        space = 8 + 4 + (10 * 32) + 1// Maximum of 10 entities
    )]
    pub trusted_roles: Account<'info, TrustedEntities>,

    pub system_program: Program<'info, System>,
 }

/*
The Whitelister context Struct */
#[derive(Accounts)]
#[instruction(whitelister_address: Pubkey)]
pub struct InitializeWhiteLister<'info> {

    #[account(
        mut,
        constraint = admin.key() == admin_account.admin_address.key() @ LendanaError::OnlyAdmin
    )]
    pub admin: Signer<'info>,

    #[account(
        seeds = [b"admin", admin_account.admin_address.key().as_ref()],
        bump = admin_account.admin_bump
    )]
    pub admin_account: Account<'info, Administrator>,

    #[account(
        mut,
        seeds = [b"trusted_entities"],
        bump = trusted_roles.trusted_entities_bump,
    )]
    pub trusted_roles: Account<'info, TrustedEntities>,

    #[account(
        init,
        payer = admin,
        seeds = [b"whitelister", whitelister_address.key().as_ref()],
        bump,
        space = 8 + 32 + 1 // 8 for discriminator, 32 for pubkey and 1 for bump
    )]
    pub whitelister: Account<'info, WhitelisterInfo>,

    pub system_program: Program<'info, System>,

}

/* A Global Container Of Whitelisted Tokens */
#[derive(Accounts)]
pub struct GlobalWhitelistedTokensPositionCountersAndPriceRegistry<'info> {
    // Signer ought to be whitelister
    #[account(
        mut,
        constraint = whitelister_role.key() == whitelister.address.key() @ LendanaError::OnlyWhitelister,
    )]
    pub whitelister_role: Signer<'info>,

    #[account(
        mut,
        seeds = [b"whitelister", whitelister_role.key().as_ref()],
        bump = whitelister.whitelister_bump,
    )]
    pub whitelister: Account<'info, WhitelisterInfo>,

    // Let's Initialize The Global Container For Whitelisted tokens Here
    #[account(
        init,
        payer = whitelister_role,
        seeds = [b"all_whitelisted_tokens"],
        bump,
        space = 8 + 4 + (7 * 32) + 1,// Support Up to 7 Tokens
    )]
    pub all_whitelisted_tokens: Account<'info, AllWhitelistedTokens>,

    // Let's Initialize The Lender Position ID Counter Here
    #[account(
        init,
        payer = whitelister_role,
        space = 8 + 8 + 1,
        seeds = [b"lenders_position_id_counter"],
        bump,
    )]
    pub lenders_position_id_counter: Account<'info, LenderPositionIDCounter>,

    // Let's Initialize The Borrower Position ID Counter Here
    #[account(
        init,
        payer = whitelister_role,
        space = 8 + 8 + 1,
        seeds = [b"borrowers_position_id_counter"],
        bump,
    )]
    pub borrowers_position_id_counter: Account<'info, BorrowerPositionIDCounter>,

    // Token Price Feeds Registry
    #[account(
        init,
        payer = whitelister_role,
        seeds = [b"price_feed_registry"],
        bump,
        space = 8 + TokenPriceFeedRegistry::INIT_SPACE,
    )]
    pub tokens_price_feed_registry: Account<'info, TokenPriceFeedRegistry>,

    // SOL COLLATERAL PDA VAULT
    #[account(
        init,
        payer = whitelister_role,
        seeds = [b"sol_collateral_vault"],
        bump,
        space = 8 + SolCollateralVault::INIT_SPACE,
    )]
    pub sol_collateral_vault: Account<'info, SolCollateralVault>,

    pub system_program: Program<'info, System>,
}
/** TOKEN WHITELISTING OPERATION */
#[derive(Accounts)]
#[instruction(token_mint: Pubkey)]
pub struct WhitelistToken<'info> {
    // Signer supposed to be whitelister
    #[account(
        mut,
        constraint = whitelister_role.key() == whitelister.address.key() @ LendanaError::OnlyWhitelister,
    )]
    pub whitelister_role: Signer<'info>,

    #[account(
        mut,
        seeds = [b"whitelister", whitelister.address.key().as_ref()],
        bump = whitelister.whitelister_bump,
    )]
    pub whitelister: Account<'info, WhitelisterInfo>,

    // Global Registry Whitelisted
    #[account(
        mut,
        seeds = [b"all_whitelisted_tokens"],
        bump = all_whitelisted_tokens.tokens_whitelisted_bump,
    )]
    pub all_whitelisted_tokens: Account<'info, AllWhitelistedTokens>,

    // Mint of Whitelisted Token
    #[account(
        constraint = token_mint.key() == mint_token.key() @LendanaError::MismatchedTokenMint,
    )]
    pub mint_token: InterfaceAccount<'info, Mint>,

    // Token Escrow Account
    #[account(
        init,
        payer = whitelister_role,
        space = 8 + 32 + 8 + 8 + 1 + 1,
        seeds = [b"token_escrow", mint_token.key().as_ref()],
        bump
    )]
    pub token_escrow: Account<'info, LentBorrowedTokenEscrow>,

    // The Associated Token Esrow Vault
    #[account(
        init,
        payer = whitelister_role,
        associated_token::mint = mint_token,
        associated_token::authority = token_escrow,
    )]
    pub token_vault: InterfaceAccount<'info, TokenAccount>,

    // System and Associated Token programs, and Token program
    pub system_program: Program<'info, System>,

    pub associated_token_program: Program<'info, AssociatedToken>,

    pub token_program: Interface<'info, TokenInterface>,
}


/* ADD A WHITELISTED TOKEN PRICE FEED MAPPING */
#[derive(Accounts)]
pub struct AddTokenPriceMapping<'info> {

    #[account(
        mut,
        constraint = whitelister.key() == tokens_price_feed_registry.authority @LendanaError::UnauthorizedPriceUpdater,
    )]
    pub whitelister: Signer<'info>,

    // Token Price Registry
    #[account(
        mut,
        seeds = [b"price_feed_registry"],
        bump = tokens_price_feed_registry.registry_bump
    )]
    pub tokens_price_feed_registry: Account<'info, TokenPriceFeedRegistry>,
}

impl<'info> AddTokenPriceMapping<'info> {
    // Method To Add Price Feed
    pub fn add_token_price_to_registry(&mut self, token_mint: Pubkey, price_feed_id_hex: String) -> Result<()> {

        let token_price_registry =&mut self.tokens_price_feed_registry;

        // Check if Price Mapping For Specified Token Does Not Exist
        if token_price_registry.token_price_mapping.iter().any(|m| m.token_mint == token_mint) {
            return err!(LendanaError::TokenPriceAlreadyExists);
        }

        // Let's Put This Token Price Feed Id into The Registry
        let price_feed_id = price_feed_id_hex.to_string();
        token_price_registry.token_price_mapping.push(TokenPriceMapping { token_mint, price_feed_id});
        Ok(())
    }
}