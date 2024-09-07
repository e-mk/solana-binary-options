
use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace)]
pub struct BinaryOption {
    pub market_maker: Pubkey,
    pub bettor: Option<Pubkey>,
    pub amount: u64,
    pub price_condition: i64,
    pub time_condition: i64,
    #[max_len(10)]
    pub condition: String,
    pub bump: u8,
    pub seed: u64,
}