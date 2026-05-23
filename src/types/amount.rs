use crate::Fee;
use thiserror::Error;

pub type Amount = rust_decimal::Decimal;

pub trait AmountExt: Sized {
    fn sub_fee(self, fee: Fee) -> Result<Self, AmountSubFeeError>;
}

impl AmountExt for Amount {
    fn sub_fee(self, fee: Fee) -> Result<Self, AmountSubFeeError> {
        use AmountSubFeeError::*;
        let amount = self;
        let multiplier = errgonomic::handle_opt!(Fee::ONE.checked_sub(fee), FeeCheckedSubFailed, fee);
        Ok(errgonomic::handle_opt!(amount.checked_mul(multiplier), AmountCheckedMulFailed, amount, multiplier))
    }
}

#[derive(Error, Copy, Clone, Debug)]
pub enum AmountSubFeeError {
    #[error("failed to subtract fee '{fee}' from one")]
    FeeCheckedSubFailed { fee: Fee },
    #[error("failed to multiply amount '{amount}' by fee multiplier '{multiplier}'")]
    AmountCheckedMulFailed { amount: Amount, multiplier: Fee },
}
