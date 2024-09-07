use std::str::FromStr;

use anchor_lang::prelude::*;
use anchor_spl::token_interface::{transfer_checked, Mint, TokenAccount, TokenInterface, TransferChecked, close_account, CloseAccount};
use pyth_solana_receiver_sdk::price_update::Price;

pub use crate::error::ErrorCode;
use crate::{BinaryOption, ADMIN_WALLET, BINARY_OPTION_SEED, HOUSE_SHARE_PERCENTAGE};

#[derive(Accounts)]
#[instruction()]
pub struct ResolveBinaryOptions<'info> {
  #[account(mut)]
  pub signer: Signer<'info>,

  #[account(address = Pubkey::from_str(ADMIN_WALLET).unwrap())]
  pub house: SystemAccount<'info>,

  #[account(
    mut,
    associated_token::mint = mint,
    associated_token::authority = house,
    associated_token::token_program = token_program
  )]
  pub house_ata: InterfaceAccount<'info, TokenAccount>,

  #[account(mut)]
  pub market_maker: SystemAccount<'info>,

  #[account(
    mut,
    associated_token::mint = mint,
    associated_token::authority = market_maker,
    associated_token::token_program = token_program
  )]
  pub market_maker_ata: InterfaceAccount<'info, TokenAccount>,

  #[account(mut)]
  pub bettor: SystemAccount<'info>,

  #[account(
    mut,
    associated_token::mint = mint,
    associated_token::authority = bettor,
    associated_token::token_program = token_program
  )]
  pub bettor_ata: InterfaceAccount<'info, TokenAccount>,

  #[account(
    mint::token_program = token_program
  )]
  pub mint: InterfaceAccount<'info, Mint>,

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

  // mainnet
  // pub price_update: Account<'info, PriceUpdateV2>,

  // devnet
  pub price_update:  SystemAccount<'info>,
  
  pub token_program: Interface<'info, TokenInterface>,
  pub system_program: Program<'info, System>,
}

impl<'info> ResolveBinaryOptions<'info> {
  pub fn resolve_options(&mut self) -> Result<()> {
    
    let clock = Clock::get()?;
    let current_time_stamp = clock.unix_timestamp * 1000;

    let binary_option = &mut self.binary_option;

    require!(
      current_time_stamp > binary_option.time_condition,
      ErrorCode::TimeConditionViolation
    );
    
    let signer_seeds: [&[&[u8]]; 1] = [&[
      BINARY_OPTION_SEED,
      self.market_maker.to_account_info().key.as_ref(),
      &binary_option.seed.to_le_bytes()[..],
      &[binary_option.bump],
  ]];

    // mainnet
    // let price_update = &mut self.price_update;
    // let price: pyth_solana_receiver_sdk::price_update::Price = price_update.get_price_no_older_than(
    //   &Clock::get()?,
    //   MAXIMUM_AGE_MINUTES,
    //   &get_feed_id_from_hex(PYTH_SOL_USD_FEED_ID)?,
    // )?;
    // devnet
    let price: pyth_solana_receiver_sdk::price_update::Price = Price {        
      price: 150,        
      conf: 0,              
      exponent: 0,          
      publish_time: 0
    };

    let admin_transfer_accounts = TransferChecked {
      from: self.vault.to_account_info(),
      mint: self.mint.to_account_info(),
      to: self.house_ata.to_account_info(),
      authority: binary_option.to_account_info(),
    };

    let cpi_ctx = CpiContext::new_with_signer(self.token_program.to_account_info(), admin_transfer_accounts, &signer_seeds);

    let house_share = self.vault.amount * HOUSE_SHARE_PERCENTAGE / 100;
    transfer_checked(cpi_ctx, house_share, self.mint.decimals)?;

    let winner_ata_account_info = if (price.price > binary_option.price_condition && binary_option.condition == "more") || (price.price < binary_option.price_condition && binary_option.condition == "less") {
      self.market_maker_ata.to_account_info()
    } else {
      self.bettor_ata.to_account_info()
    };

    let winner_transfer_accounts = TransferChecked {
      from: self.vault.to_account_info(),
      mint: self.mint.to_account_info(),
      to: winner_ata_account_info,
      authority: binary_option.to_account_info(),
    };

    let cpi_ctx = CpiContext::new_with_signer(self.token_program.to_account_info(), winner_transfer_accounts, &signer_seeds);

    transfer_checked(cpi_ctx, self.vault.amount - house_share, self.mint.decimals)?;

    let accounts = CloseAccount {
      account: self.vault.to_account_info(),
      destination: binary_option.to_account_info(),
      authority: binary_option.to_account_info(),
    };

    let ctx = CpiContext::new_with_signer(
        self.token_program.to_account_info(),
        accounts,
        &signer_seeds,
    );

    close_account(ctx)
  }
}