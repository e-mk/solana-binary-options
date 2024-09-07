use anchor_lang::prelude::*;

#[constant]
pub const ADMIN_WALLET: &str = "FexVSkWTomAziTdHs2E4FyDougLbCEZXjHLn4vaPhmiB";
pub const HOUSE_SHARE_PERCENTAGE: u64 = 1; 

pub const THREAD_AUTHORITY_SEED: &[u8] = b"authority";
pub const BINARY_OPTION_SEED: &[u8] = b"binary_options";

pub const MAXIMUM_AGE_MINUTES: u64 = 60; 
pub const PYTH_SOL_USD_FEED_ID: &str = "0xef0d8b6fda2ceba41da15d4095d1da392a0d2f8ed0c6c7bc0f4cfac8c280b56d";
