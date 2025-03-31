use anchor_lang::prelude::*;

use crate::{states::contexts::*, LentBorrowedTokenEscrow};


pub fn token_whitelist(ctx: Context<WhitelistToken>, token_mint: Pubkey) -> Result<()> {

    // Set the lent token vault details
    let lent_tokens = &mut ctx.accounts.token_escrow;
    lent_tokens.set_inner(LentBorrowedTokenEscrow{
        lending_borrowing_token: ctx.accounts.mint_token.key(),
        total_lent_tokens: 0,
        total_borrowed_tokens: 0,
        is_active: true,
        token_vault_bump: ctx.bumps.token_escrow
    });

    let all_tokens = &mut ctx.accounts.all_whitelisted_tokens;
    all_tokens.tokens_whitelisted.push(token_mint);
    Ok(())
}