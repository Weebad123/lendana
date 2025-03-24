use anchor_lang::prelude::*;

use crate::states::{contexts::*, errors::*};



pub fn admin_initialize(ctx: Context<InitializeAdmin>, admin_address: Pubkey) -> Result<()> {

    // Initialize Admin Account
    let admin_info = &mut ctx.accounts.admin_account;

    admin_info.admin_address = admin_address;
    admin_info.admin_bump = ctx.bumps.admin_account;

    Ok(())
}