use anchor_lang::prelude::*;

#[error_code]
pub enum ErrorCode {
  #[msg("Given binary option condition is invalid")]
  InvalidBinaryOptionCondition,

  #[msg("Insufficient token balance for the bet")]
  InsufficientBalanceForBet,

  #[msg("Can't resolve binary option before given time condition")]
  TimeConditionViolation,
}
