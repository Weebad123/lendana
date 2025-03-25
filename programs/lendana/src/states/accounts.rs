use anchor_lang::prelude::*;


/**
 * We Need An Administrator Role
 */

 #[account]
 pub struct Administrator {
    pub admin_address: Pubkey,
    pub admin_bump: u8
 }

 /*
 A Global Registry Of Trusted Roles */
 #[account]
 pub struct TrustedEntities {
    pub trusted_roles: Vec<Pubkey>,
    pub trusted_entities_bump: u8,
 }
/* 
The platform will support only whitelisted tokens, so we need to first give the whitelister role */

#[account]
pub struct WhitelisterInfo {
    pub address: Pubkey,
    pub whitelister_bump: u8
}

/** A global Mapping Of Whitelisted Tokens */
#[account]
pub struct AllWhitelistedTokens {
   pub tokens_whitelisted: Vec<Pubkey>,
   pub tokens_whitelisted_bump: u8,
}


/* Lender Position ID Counter */
#[account]
pub struct LenderPositionIDCounter {
   pub lenders_current_position_id: u64,

   pub lender_position_id_bump: u8
}


/* Borrower Position ID Counter */
#[account]
pub struct BorrowerPositionIDCounter {
   pub borrowers_current_position_id: u64,

   pub borrower_position_id_bump: u8,
}


/** An Associated Token Vault To Hold All Lent Tokens */
#[account]
pub struct LentTokenVault {

   pub lending_token: Pubkey,
   
   pub total_lent_tokens: u64,

   pub token_vault_bump: u8,

   pub is_active: bool,
}





/* -------------------------------------               LENDER ACCOUNTS              ------------------------------------ */



/* THE LENDER POSITION */
#[account]
#[derive(InitSpace)]
pub struct LenderPosition {
   
   pub lending_token: Pubkey,// 32 bytes

   pub lender_pubkey: Pubkey,

   pub lending_amount: u64,// 8 bytes

   pub interest_accumulated: u64,// 8 bytes

   pub lender_position_id: u64,// 8 bytes

   pub lending_terms: LoanTerms,// 16 bytes

   pub is_position_active: bool,// 1 byte

   pub is_matched: bool,// 1 byte

   pub lending_start: i64,// 8 bytes

   pub lender_position_bump: u8,// 1 byte

}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, InitSpace, PartialEq)]
pub struct LoanTerms {

   pub interest_rate: u64,

   pub lending_duration: u64,
}