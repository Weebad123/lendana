use anchor_lang::prelude::*;

use crate::states::contexts::*;


pub fn initialize_trusted_entities(ctx: Context<InitializeTrustedRoles>) -> Result<()> {

    let global_trusted = &mut ctx.accounts.trusted_roles;
    global_trusted.trusted_roles = Vec::new();
    global_trusted.trusted_entities_bump = ctx.bumps.trusted_roles;
    Ok(())
}