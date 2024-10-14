use anchor_lang::prelude::*;

#[error_code]
pub enum ErrorCode {
    #[msg("You are not authorized to perform this action.")]
    Unauthorized, // 0x1770
    #[msg("InvalidInstruction")]
    Invalid, // 0x1771
    #[msg("The config has already been initialized.")]
    ReInitialize, // 0x1772
    #[msg("The config has not been initialized.")]
    UnInitialize, // 0x1773
    #[msg("Argument is invalid.")]
    InvalidArgument, // 0x1774
    #[msg("An overflow occurs.")]
    Overflow, // 0x1775
    #[msg("Pyth has an internal error.")]
    PythError, // 0x1776
    #[msg("Pyth price oracle is offline.")]
    PythOffline, // 0x1777
    #[msg("Program should not try to serialize a price account.")]
    TryToSerializePriceAccount, // 0x1778
    #[msg("You can't borrow an asset you deposited : start by withdrawing the asset you want to borrow")]
    NoBorrowIfDeposited, // 0x1779
    #[msg("You're not supposed to be able to redeem")]
    NoBorrowWantsToRedeem, // 0x1779
    #[msg("Lets work with bigger amounts")]
    AmountTooLow, // 0x1779
    #[msg("Lets work with lower amounts")]
    AmountTooBig, // 0x1779

}
