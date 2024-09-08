use anchor_lang::prelude::*;
use anchor_spl::token_interface::{transfer_checked, Mint, TokenAccount, TokenInterface, TransferChecked};
pub use crate::error::ErrorCode;

use crate::{BinaryOption, BINARY_OPTION_SEED};

#[derive(Accounts)]
#[instruction()]
pub struct DoBet<'info> {
  
  #[account(mut)]
  pub bettor: Signer<'info>,

  #[account(mut)]
  pub market_maker: SystemAccount<'info>,

  #[account(
    mint::token_program = token_program
  )]
  pub mint: InterfaceAccount<'info, Mint>,

  #[account(
    mut,
    associated_token::mint = mint,
    associated_token::authority = bettor,
    associated_token::token_program = token_program
  )]
  pub bettor_ata: InterfaceAccount<'info, TokenAccount>,

  #[account(
    mut,
    owner = crate::ID,
    seeds = [BINARY_OPTION_SEED, market_maker.key().as_ref(), binary_option.seed.to_le_bytes().as_ref()],
    bump = binary_option.bump
  )]
  pub binary_option: Account<'info, BinaryOption>,
  #[account(
    mut,
    associated_token::mint = mint,
    associated_token::authority = binary_option,
    associated_token::token_program = token_program,
  )]
  pub vault: InterfaceAccount<'info, TokenAccount>,

  // devnet / mainnet
  // pub price_update: Account<'info, PriceUpdateV2>,

  // localnet
  pub price_update:  SystemAccount<'info>,

  pub token_program: Interface<'info, TokenInterface>,
  pub system_program: Program<'info, System>
}

impl<'info> DoBet<'info> {
  pub fn save_bet(&mut self) -> Result<()> {

    require!(
      self.bettor_ata.amount >= self.binary_option.amount,
      ErrorCode::InsufficientBalanceForBet
    );

    let binary_option = &mut self.binary_option;
    binary_option.bettor = Option::Some(self.bettor.key());

    let transfer_accounts = TransferChecked {
      from: self.bettor_ata.to_account_info(),
      mint: self.mint.to_account_info(),
      to: self.vault.to_account_info(),
      authority: self.bettor.to_account_info(),
    };

    let cpi_ctx = CpiContext::new(self.token_program.to_account_info(), transfer_accounts);

    transfer_checked(cpi_ctx, self.binary_option.amount, self.mint.decimals)
  }
}