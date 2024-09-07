
use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace)]
pub struct Bet {
    pub bettor: Pubkey,
    pub amount: u64,
    pub bump: u8,
    pub seed: u64,
}