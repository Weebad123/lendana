// Some Constants To Be Used And Their Limits

use anchor_spl::token::spl_token;
use anchor_lang::prelude::*;


pub const MAX_ALLOWABLE_INTEREST_RATE_BPS: u64 = 700;// In Basis Points

pub const SIX_MONTH_LENDING_DURATION: u64 = 15_552_000;// 86400 * 180

pub const THREE_MONTH_LENDING_DURATION: u64 = 7_776_000;// 86400 * 90

pub const ONE_MONTH_LENDING_DURATION: u64 = 2_592_000;// 86400 * 30

pub const MIN_COLLATERAL_RATIO: u64 = 12000; // 150%

// Wrapped SOL Mint Address
pub const NATIVE_SOL_MINT_ADDRESS: Pubkey = spl_token::native_mint::id();


// Some Price Feeds IDs from Pyth Network Oracle for Supported Whitelisted Tokens
// ETH / USD price feed ID
pub const ETH_USD_PRICE_FEED_ID_HEX: &str = "0xff61491a931112ddf1bd8147cd1b641375f79f5825126d665480874634fd0ace";

// BTC / USD price feed ID
pub const BTC_USD_PRICE_FEED_ID_HEX: &str = "0xe62df6c8b4a85fe1a67db44dc12de5db330f7ac66b72dc658afedf0f4a415b43";

// USDC / USD price feed ID
pub const USDC_USD_PRICE_FEED_ID_HEX: &str = "0xeaa020c61cc479712813461ce153894a96a6c00b21ed0cfc2798d1f9a9e9c94a";

// SOL / USD price feed ID
pub const SOL_USD_PRICE_FEED_ID_HEX: &str = "0xef0d8b6fda2ceba41da15d4095d1da392a0d2f8ed0c6c7bc0f4cfac8c280b56d";

// JITOSOL / USD price feed ID
pub const JITOSOL_USD_PRICE_FEED_ID_HEX: &str = "0x67be9f519b95cf24338801051f9a808eff0a578ccb388db73b7f6fe1de019ffb";

// MAXIMUM AGE OF PRICE FEED
pub const MAX_PRICE_FEED_AGE: u64 = 30; // 30 seconds