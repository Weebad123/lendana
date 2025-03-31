use anchor_lang::prelude::*;


use crate::states::contexts::*;


pub fn add_token_prices(ctx: Context<AddTokenPriceMapping>, token_mint: Pubkey, price_feed_id: String) -> Result<()> {

    // Let's Call The Method
    ctx.accounts.add_token_price_to_registry(token_mint, price_feed_id)?;
    Ok(())
}