pub mod constants;
pub mod error;
pub mod instructions;
pub mod state;

use anchor_lang::prelude::*;

pub use constants::*;
pub use instructions::*;
pub use state::*;

declare_id!("FWLftmf5jaj372ZzNWnQVhMUe27R6nfuR9xkbwb6MGCG");

#[program]
pub mod binary_options_test_task {

    use super::*;

    pub fn initialize(
      ctx: Context<Initialize>,
      seed: u64,
      amount: u64,
      condition: String,
      price_condition: i64, 
      time_condition: i64
  ) -> Result<()> {
    ctx.accounts.create_binary_option(amount, condition, price_condition, time_condition, seed, &ctx.bumps)
  }

  pub fn do_bet(
    ctx: Context<DoBet>
  ) -> Result<()> {
    ctx.accounts.save_bet()
  }

  pub fn resolve(
    ctx: Context<ResolveBinaryOptions>
  ) -> Result<()> {
    ctx.accounts.resolve_options()
  }
}
