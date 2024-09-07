use anchor_lang::prelude::*;
use anchor_spl::{
  associated_token::AssociatedToken,
  token_interface::{transfer_checked, Mint, TokenAccount, TokenInterface, TransferChecked},
};
pub use crate::error::ErrorCode;

use crate::{BinaryOption, BINARY_OPTION_SEED};

#[derive(Accounts)]
#[instruction(seed: u64, amount: u64, condition: String, price_condition: u64, time_condition: i64)]
pub struct Initialize<'info> {
  #[account(mut)]
  pub authority: Signer<'info>,

  #[account(
    mint::token_program = token_program
  )]
  pub mint: InterfaceAccount<'info, Mint>,

  #[account(
    mut,
    associated_token::mint = mint,
    associated_token::authority = authority,
    associated_token::token_program = token_program
  )]
  pub market_maker_ata: InterfaceAccount<'info, TokenAccount>,

  #[account(
    init,
    payer = authority,
    space = 8 + BinaryOption::INIT_SPACE,
    seeds = [BINARY_OPTION_SEED, authority.key().as_ref(), seed.to_le_bytes().as_ref()],
    bump
  )]
  pub binary_option: Account<'info, BinaryOption>,

  #[account(
    init,
    payer = authority,
    associated_token::mint = mint,
    associated_token::authority = binary_option,
    associated_token::token_program = token_program
  )]
  pub vault: InterfaceAccount<'info, TokenAccount>,

  pub associated_token_program: Program<'info, AssociatedToken>,
  pub token_program: Interface<'info, TokenInterface>,
  pub system_program: Program<'info, System>,
}

impl<'info> Initialize<'info> {
  pub fn create_binary_option(&mut self, amount: u64, condition: String, price_condition: i64, time_condition: i64, seed: u64, bumps: &InitializeBumps) -> Result<()> {

    require!(
      condition != "less" || condition != "more",
      ErrorCode::InvalidBinaryOptionCondition
    );

    let clock = Clock::get()?;
    let current_time_stamp = clock.unix_timestamp * 1000;

    require!(
      current_time_stamp < time_condition,
      ErrorCode::TimeConditionViolation
    );

    self.binary_option.set_inner(BinaryOption {
        seed,
        market_maker: self.authority.key(),
        bettor: None,
        amount,
        condition,
        price_condition,
        time_condition,
        bump: bumps.binary_option,
    });

    let transfer_accounts = TransferChecked {
      from: self.market_maker_ata.to_account_info(),
      mint: self.mint.to_account_info(),
      to: self.vault.to_account_info(),
      authority: self.authority.to_account_info(),
    };

    let cpi_ctx = CpiContext::new(self.token_program.to_account_info(), transfer_accounts);

    transfer_checked(cpi_ctx, amount, self.mint.decimals)
  }
}