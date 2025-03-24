use anchor_lang::prelude::*;

use crate::states::{contexts::*, errors::*};


pub fn initialize_whitelister(ctx: Context<InitializeWhiteLister>, whitelister_address: Pubkey) -> Result<()> {

    // Get Whitelister Info, and Global Trusted , and Instantiates it below
    let whitelister_info = &mut ctx.accounts.whitelister;

    whitelister_info.address = whitelister_address;
    whitelister_info.whitelister_bump = ctx.bumps.whitelister;

    let global_trusted = &mut ctx.accounts.trusted_roles;
    // So we add the whitelister PDA to the trusted Authorities
    global_trusted.trusted_roles.push(whitelister_info.key());
    Ok(())
}