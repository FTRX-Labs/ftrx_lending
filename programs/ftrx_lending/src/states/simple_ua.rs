use anchor_lang::prelude::*;
use crate::states::{simple_pool::SimplePool};
//cargo update -p solana-program@2.0.9 --precise ver
#[account]
#[derive(Default)]
pub struct SimpleUa{
    pub ua_bump:u8,
    pub pool_pk:Pubkey,
    pub ua_authority:Pubkey,
    pub user_stable_share_deposited:u64,
    pub user_volatile_share_deposited:u64,
    pub user_stable_share_borrowed:u64,
    pub user_volatile_share_borrowed:u64,

    pub stable_c_token_minted:u64,
    pub volatile_c_token_minted:u64,
    pub ftrx_usdc_minted:u64,
    pub ftrx_volatile_minted:u64,
    pub stable_c_token_locked:u64,
    pub volatile_c_toke_locked:u64

}


impl SimpleUa{


    pub fn get_max_new_borrowable_volatile_share(
        &mut self,
        simple_pool:&SimplePool,
        price_pool:i64,
        price_multiplier:u64,
        )
        -> Option<(u64)>{
            let BASE_MULTIPLIER:u64=1_000_000;
            let SHARE_VALUE_ADAPTER:u64=1_000_000_000_000;
            let REAL_SHARE_VALUE_ADAPTER:u64=1_000_000_000_000;
            let top_yield_one:u128=60;
            let long_term_yield:u128=100;
            let usdc_amount=self.user_stable_share_deposited.checked_mul(simple_pool.stable_share_asset_value).unwrap().checked_div(SHARE_VALUE_ADAPTER).unwrap();
            msg!["LOG USDC amount collateral {}/1000000",usdc_amount];
            let max_volatile_value_new_borrow=usdc_amount.checked_mul(simple_pool.new_borrow_max_ltv).unwrap().checked_div(BASE_MULTIPLIER).unwrap();
            msg!["LOG USDC amount to borrow with stable {}/1000000 ",max_volatile_value_new_borrow];
            let max_volatile_amount_new_borrow=max_volatile_value_new_borrow.checked_mul(price_multiplier).unwrap().checked_div(price_pool as u64).unwrap();
            msg!["LOG Volatile amount to borrow with stable {}/1000000 ",max_volatile_amount_new_borrow ];
            let max_volatile_share_new_borrow=max_volatile_amount_new_borrow.checked_mul(REAL_SHARE_VALUE_ADAPTER).unwrap().checked_div(simple_pool.volatile_share_liabi_value).unwrap();
            msg!["LOG Volatile share to borrow with stable {}/1000000 ",max_volatile_share_new_borrow ];
            
            //Amount with 6 decimals, needs to be adapted later
            Some(max_volatile_share_new_borrow)
    }

    pub fn get_max_maintainance_borrowable_volatile(
        &mut self,
        simple_pool:&SimplePool,
        price_pool:i64,
        price_multiplier:u64,
        )
        -> Option<(u64)>{
            let BASE_MULTIPLIER:u64=1_000_000;
            let SHARE_VALUE_ADAPTER:u64=1_000_000_000_000;
            let top_yield_one:u128=60;
            let long_term_yield:u128=100;
            let usdc_amount=self.user_stable_share_deposited.checked_mul(simple_pool.stable_share_asset_value).unwrap().checked_div(SHARE_VALUE_ADAPTER).unwrap();
            msg!["LOG USDC amount collateral {}/1000000 {}",usdc_amount,self.user_stable_share_deposited];
            let max_volatile_value_maintainance_borrow=usdc_amount.checked_mul(simple_pool.liquidation_ltv).unwrap().checked_div(BASE_MULTIPLIER).unwrap();
            msg!["LOG USDC amount to borrow with stable {}/1000000",max_volatile_value_maintainance_borrow];
            let max_volatile_amount_maintainance_borrow=usdc_amount.checked_mul(price_multiplier).unwrap().checked_div(price_pool as u64).unwrap();
            msg!["LOG Volatile max amount to borrow with stable {}/1000000",max_volatile_amount_maintainance_borrow];
            let max_volatile_share_maintainance_borrow=max_volatile_amount_maintainance_borrow.checked_mul(SHARE_VALUE_ADAPTER).unwrap().checked_div(simple_pool.volatile_share_liabi_value).unwrap();
            msg!["LOG Volatile max share to borrow with stable {}/1000000",max_volatile_share_maintainance_borrow];
            
            Some(max_volatile_share_maintainance_borrow)
    }

    pub fn get_max_new_borrowable_stable(
        &mut self,
        simple_pool:&SimplePool,
        price_pool:i64,
        price_multiplier:u64,
        )
        -> Option<(u64)>{
            let BASE_MULTIPLIER:u64=1_000_000;
            let SHARE_VALUE_ADAPTER:u64=1_000_000_000_000;
            let top_yield_one:u128=60;
            let long_term_yield:u128=100;
            msg!["LOGuser volatile_share deposited {}/1000000",self.user_volatile_share_deposited];
            let volatile_amount=self.user_volatile_share_deposited.checked_mul(simple_pool.volatile_share_asset_value).unwrap().checked_div(SHARE_VALUE_ADAPTER).unwrap();
            msg!["LOG Volatile amount collateral {}/1000000",volatile_amount];
            let volatile_value=volatile_amount.checked_mul(price_pool  as u64).unwrap().checked_div(price_multiplier).unwrap();
            msg!["LOG Volatile value collateral {}/1000000",volatile_value];
            
            let max_stable_value_new_borrow=volatile_value.checked_mul(simple_pool.new_borrow_max_ltv).unwrap().checked_div(BASE_MULTIPLIER).unwrap();;
            msg!["LOG USDC amount to borrow with stable {}/1000000",max_stable_value_new_borrow];
            let max_stable_amount_new_borrow=max_stable_value_new_borrow;
            msg!["LOG USDC amount to borrow with stable {}/1000000",max_stable_amount_new_borrow];
            let max_stable_share_new_borrow=max_stable_amount_new_borrow.checked_mul(SHARE_VALUE_ADAPTER).unwrap().checked_div(simple_pool.stable_share_liabi_value).unwrap();
            msg!["LOG USDC share to borrow with stable {}/1000000 ",max_stable_share_new_borrow ];
            
            Some(max_stable_amount_new_borrow)
    }

    pub fn get_max_maintainance_borrowable_stable(
        &mut self,
        simple_pool:&SimplePool,
        price_pool:i64,
        price_multiplier:u64,
        )
        -> Option<(u64)>{
            let BASE_MULTIPLIER:u64=1_000_000;
            let SHARE_VALUE_ADAPTER:u64=1_000_000_000_000;
            let top_yield_one:u128=60;
            let long_term_yield:u128=100;
            let volatile_amount=self.user_volatile_share_deposited.checked_mul(simple_pool.volatile_share_asset_value).unwrap().checked_div(SHARE_VALUE_ADAPTER).unwrap();
            msg!["LOG Volatile amount collateral {}/1000000",volatile_amount];
            let volatile_value=volatile_amount.checked_mul(price_pool as u64).unwrap().checked_div(price_multiplier).unwrap();
            msg!["LOG Volatile value collateral {}/1000000",volatile_value];
            
            let max_stable_value_maintainance_borrow=volatile_value.checked_mul(simple_pool.new_borrow_max_ltv).unwrap().checked_div(BASE_MULTIPLIER).unwrap();;
            msg!["LOG USDC max value to borrow with volatile {}/1000000",max_stable_value_maintainance_borrow];
            let max_stable_amount_maintainance_borrow=max_stable_value_maintainance_borrow;
            msg!["LOG USDC max amount to borrow with volatile {}/1000000",max_stable_amount_maintainance_borrow];
            let max_stable_share_maintainance_borrow=max_stable_value_maintainance_borrow.checked_mul(SHARE_VALUE_ADAPTER).unwrap().checked_div(simple_pool.stable_share_liabi_value).unwrap();
            msg!["LOG USDC max share to borrow with volatile {}/1000000",max_stable_share_maintainance_borrow];
            Some(max_stable_share_maintainance_borrow)
    }

}


