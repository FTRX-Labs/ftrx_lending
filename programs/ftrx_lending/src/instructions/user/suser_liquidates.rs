//libraries
use anchor_lang::prelude::*;
use pyth_solana_receiver_sdk::price_update::{PriceUpdateV2,get_feed_id_from_hex};
use std::mem::size_of;
use anchor_spl::token::{self, Burn, Mint, Token, TokenAccount, Transfer};
//local imports

use crate::states::{simple_pool::SimplePool};
use crate::states::{simple_ua::SimpleUa};
use crate::errors::ErrorCode;

pub fn handle(
    ctx: Context<SUserLiquidates>,
    asset_index: u8,
    share_amount:u64,
)->Result<()>{

  let price_update = &mut ctx.accounts.pyth_feed;
  let simple_pool = &mut ctx.accounts.simple_pool;
  let mut user_to_liquidate =&mut ctx.accounts.user_to_liquidate_state;
  let mut liquidator_account = &mut ctx.accounts.user_state;
  let SHARE_VALUE_MULTIPLIER:u64=1_000_000_000_000;
    // get_price_no_older_than will fail if the price update is more than 30 seconds old
    let maximum_age: u64 = 300;
    // get_price_no_older_than will fail if the price update is for a different price feed.
    // This string is the id of the BTC/USD feed. See https://pyth.network/developers/price-feed-ids for all available IDs.
    let feed_id: [u8; 32] = get_feed_id_from_hex("ef0d8b6fda2ceba41da15d4095d1da392a0d2f8ed0c6c7bc0f4cfac8c280b56d")?;
    msg!("Is the price found?");
    let price = price_update.get_price_no_older_than(&Clock::get()?, maximum_age, &feed_id)?;
    
    msg!("The price is ({} ± {}) * 10^{}", price.price, price.conf, price.exponent);
    simple_pool.last_price=price.price;
    let base: f64 = 10.0;
    let multiplier= base.powf(-price.exponent as f64) as u64;
    
   
    if asset_index==0{
      let mut max_maintainance_amount_stable=user_to_liquidate.get_max_maintainance_borrowable_stable(&simple_pool,price.price,multiplier).unwrap();
      max_maintainance_amount_stable=max_maintainance_amount_stable.checked_mul(1000).unwrap().checked_div(1200).unwrap();     
      
      msg!["LOG USDC max share to borrow with volatile vs currently_borrowed {}/1000000 {}/1000000 ",max_maintainance_amount_stable,user_to_liquidate.user_stable_share_borrowed];
      if (user_to_liquidate.user_stable_share_borrowed>max_maintainance_amount_stable){
        msg!["WE LIQUIDATE USDC debt"];
      
        let amount_share_to_liquidate=user_to_liquidate.user_stable_share_borrowed.checked_sub(max_maintainance_amount_stable).unwrap();
        msg!["Amount_share to liquidate {}/1000000",amount_share_to_liquidate];
        let amount_dollar_to_liquidate=amount_share_to_liquidate.checked_mul(simple_pool.stable_share_liabi_value).unwrap().checked_div(SHARE_VALUE_MULTIPLIER).unwrap();
        msg!["Amount USD to liquidate {}/1000000",amount_dollar_to_liquidate];
        let minimal_collateral_dollar_to_transfer=amount_dollar_to_liquidate;

        let minimal_collateral_volatile_share_to_transfer=minimal_collateral_dollar_to_transfer.checked_mul(multiplier).unwrap().checked_div(price.price as u64).unwrap().checked_mul(SHARE_VALUE_MULTIPLIER).unwrap().checked_div(simple_pool.volatile_share_asset_value).unwrap();
        msg!["Amount SOL collateral share received by liquidator {}/1000000",minimal_collateral_volatile_share_to_transfer];
        let corresponding_stable_share_to_pay=amount_share_to_liquidate;
        msg!["Amount USD debt share received by liquidator {}/1000000",corresponding_stable_share_to_pay];
        
        user_to_liquidate.user_volatile_share_deposited=user_to_liquidate.user_volatile_share_deposited.checked_sub(minimal_collateral_volatile_share_to_transfer).unwrap();
        user_to_liquidate.user_stable_share_borrowed=user_to_liquidate.user_stable_share_borrowed.checked_sub(corresponding_stable_share_to_pay).unwrap();
        
        liquidator_account.user_volatile_share_deposited=liquidator_account.user_volatile_share_deposited.checked_add(minimal_collateral_volatile_share_to_transfer).unwrap();
        liquidator_account.user_stable_share_borrowed=liquidator_account.user_stable_share_borrowed.checked_add(corresponding_stable_share_to_pay).unwrap();

      }
    }else{

      let mut  max_maintainance_amount_volatile=user_to_liquidate.get_max_maintainance_borrowable_volatile(&simple_pool,price.price,multiplier).unwrap();
      max_maintainance_amount_volatile=max_maintainance_amount_volatile.checked_mul(1000).unwrap().checked_div(1500).unwrap();     
      msg!["LOG Volatile max share to borrow with stable vs currently_borrowed {}/1000000 {}/1000000 ",max_maintainance_amount_volatile,user_to_liquidate.user_volatile_share_borrowed];
      
      if (user_to_liquidate.user_volatile_share_borrowed>max_maintainance_amount_volatile){
        msg!["WE LIQUIDATE SOL DEBT"];
        let amount_share_to_liquidate=user_to_liquidate.user_volatile_share_borrowed.checked_sub(max_maintainance_amount_volatile).unwrap();
        msg!["Amount_share to liquidate {}/1000000",amount_share_to_liquidate];
        let amount_dollar_to_liquidate=amount_share_to_liquidate.checked_mul(simple_pool.volatile_share_liabi_value).unwrap().checked_div(SHARE_VALUE_MULTIPLIER).unwrap().checked_mul(price.price as u64).unwrap().checked_div(multiplier).unwrap();
        msg!["Amount USD to liquidate {}/1000000",amount_dollar_to_liquidate];
        let minimal_collateral_dollar_to_transfer=amount_dollar_to_liquidate;


        let minimal_collateral_stable_share_to_transfer=minimal_collateral_dollar_to_transfer.checked_mul(SHARE_VALUE_MULTIPLIER).unwrap().checked_div(simple_pool.stable_share_asset_value).unwrap();
        let corresponding_volatile_borrowed_share_to_pay=amount_share_to_liquidate;
        
        user_to_liquidate.user_stable_share_deposited=user_to_liquidate.user_stable_share_deposited.checked_sub(minimal_collateral_stable_share_to_transfer).unwrap();
        user_to_liquidate.user_volatile_share_borrowed=user_to_liquidate.user_volatile_share_borrowed.checked_sub(corresponding_volatile_borrowed_share_to_pay).unwrap();
        
        liquidator_account.user_stable_share_deposited=liquidator_account.user_stable_share_deposited.checked_add(minimal_collateral_stable_share_to_transfer).unwrap();
        liquidator_account.user_volatile_share_borrowed=liquidator_account.user_volatile_share_borrowed.checked_add(corresponding_volatile_borrowed_share_to_pay).unwrap();

        let check_borrowed_volatile_value_user_to_liq=user_to_liquidate.user_volatile_share_borrowed.checked_mul(price.price as u64).unwrap().checked_div(multiplier).unwrap();
        msg!["Post liquidation : user_to liquidate collat nd volatile borrowed nd value  {}/1000000 {}/1000000 {}/1000000",user_to_liquidate.user_stable_share_deposited,user_to_liquidate.user_volatile_share_borrowed,check_borrowed_volatile_value_user_to_liq];
      }

    }

    Ok(())
}


#[derive(Accounts)]
pub struct SUserLiquidates<'info> {
    // Super User
    // Super User
    #[account(mut,
      seeds = [stable_mint.key().as_ref(),volatile_mint.key().as_ref(), simple_pool.pool_admin.key().as_ref()],
      bump,
    )]
    pub simple_pool: Box<Account<'info, SimplePool>>,


    #[account(mut,
      seeds = [simple_pool.key().as_ref(), user_state.ua_authority.key().as_ref()],
      bump,
      constraint=user_state.ua_authority.key()==user_signer.key()
      )]
      pub user_state: Box<Account<'info, SimpleUa>>,

    #[account(mut)]
    pub user_to_liquidate_state: Box<Account<'info, SimpleUa>>,


    #[account(mut)]
    pub user_signer: Signer<'info>,


    #[account(
        seeds = [b"share",simple_pool.volatile_mint.key().as_ref(),simple_pool.key().as_ref()],
        bump,
    )]
    pub volatile_share_mint: Box<Account<'info, Mint>>,


    #[account(
      seeds = [b"share",simple_pool.stable_mint.key().as_ref(),simple_pool.key().as_ref()],
      bump,
      
    )]
    pub stable_share_mint: Box<Account<'info, Mint>>,


    #[account(
        seeds = [b"vault",simple_pool.stable_mint.key().as_ref(),simple_pool.key().as_ref()],
        bump,
        
      )]
    pub stable_vault: Box<Account<'info, TokenAccount>>,


    #[account(
      seeds = [b"vault",simple_pool.volatile_mint.key().as_ref(),simple_pool.key().as_ref()],
      bump,
      
    )]
    pub volatile_vault: Box<Account<'info, TokenAccount>>,


    #[account(
      constraint = stable_mint.key() == simple_pool.stable_mint.key())]
    pub stable_mint: Box<Account<'info, Mint>>,
    #[account(
      constraint = volatile_mint.key() == simple_pool.volatile_mint.key())]
    pub volatile_mint: Box<Account<'info, Mint>>,



    
    #[account(
      constraint=pyth_feed.key()==simple_pool.pyth_feed.key()
    )]
    pub pyth_feed: Account<'info, PriceUpdateV2>,


    // Programs and Sysvars
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub rent: Sysvar<'info, Rent>,
}

