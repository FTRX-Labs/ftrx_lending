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
    ctx: Context<SUserBorrows>,
    asset_index: u8,
    asset_amount:u64,
)->Result<()>{
  let SHARE_VALUE_MULTIPLIER:u64=1_000_000_000_000;
  let price_update = &mut ctx.accounts.pyth_feed;
  let simple_pool = &mut ctx.accounts.simple_pool;
  let mut user_account = &mut ctx.accounts.user_state;
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
      if (user_account.user_stable_share_deposited>0) {return err!(ErrorCode::NoBorrowIfDeposited); }
      if (asset_amount<10_000) {return err!(ErrorCode::AmountTooLow); }
      if (asset_amount>1_000_000_000) {return err!(ErrorCode::AmountTooBig); }

      let max_amount_to_borrow_stable_share=user_account.get_max_new_borrowable_stable(&simple_pool,price.price,multiplier).unwrap();
      let stable_share_borrowed_requested=asset_amount.checked_mul(SHARE_VALUE_MULTIPLIER).unwrap().checked_div(simple_pool.stable_share_liabi_value).unwrap();
      let total_borrow_stable_share=stable_share_borrowed_requested.checked_add(user_account.user_stable_share_borrowed).unwrap();
      if max_amount_to_borrow_stable_share>total_borrow_stable_share{
          msg!["Borrow stable success"];

          
          let signer_seeds: &[&[&[u8]]] = &[&[
            simple_pool.stable_mint.as_ref(),
            simple_pool.volatile_mint.as_ref(),
            simple_pool.pool_admin.as_ref(),
            &[simple_pool.pool_bump],
        ]];

        let cpi_accounts = Transfer {
            from: ctx.accounts.stable_vault.to_account_info(),
            to: ctx.accounts.user_stable_vault.to_account_info(),
            authority: simple_pool.to_account_info(),
        };
        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer_seeds);
        token::transfer(cpi_ctx, asset_amount)?;



        let mut simple_pool = &mut ctx.accounts.simple_pool;
        simple_pool.stable_share_borrowed=simple_pool.stable_share_borrowed.checked_add(stable_share_borrowed_requested).unwrap();
        //simple_pool.volatile_deposited=simple_pool.volatile_deposited.checked_add(asset_amount).unwrap();
        user_account.user_stable_share_borrowed=user_account.user_stable_share_borrowed.checked_add(stable_share_borrowed_requested).unwrap();
  
      }

    }else{
      if (user_account.user_volatile_share_deposited>0) {return err!(ErrorCode::NoBorrowIfDeposited); }
      if (asset_amount<10_000) {return err!(ErrorCode::AmountTooLow); }
      if (asset_amount>1_000_000_000) {return err!(ErrorCode::AmountTooBig); }
      msg!["LOG requesting volatile borrow for amount {} ",asset_amount ];
            
      let max_amount_to_borrow_volatile_share=user_account.get_max_new_borrowable_volatile_share(&simple_pool,price.price,multiplier).unwrap();
      
      let adapted_volatile_asset_amount=asset_amount.checked_div(1_000).unwrap();
      let requested_volatile_share=adapted_volatile_asset_amount.checked_mul(SHARE_VALUE_MULTIPLIER).unwrap().checked_div(simple_pool.volatile_share_liabi_value).unwrap();
      msg!["LOG requesting volatile borrow for share {} ",requested_volatile_share ];
             
      let total_borrow_volatile_share=requested_volatile_share.checked_add(user_account.user_volatile_share_borrowed).unwrap();
      msg!["LOG would lead to a total volatile borrow for share {} ",total_borrow_volatile_share ];
             
      if max_amount_to_borrow_volatile_share>total_borrow_volatile_share{
      msg!["Borrow volatile success"];



      let signer_seeds: &[&[&[u8]]] = &[&[
          simple_pool.stable_mint.as_ref(),
          simple_pool.volatile_mint.as_ref(),
          simple_pool.pool_admin.as_ref(),
          &[simple_pool.pool_bump],
      ]];

      let cpi_accounts = Transfer {
          from: ctx.accounts.volatile_vault.to_account_info(),
          to: ctx.accounts.user_volatile_vault.to_account_info(),
          authority: simple_pool.to_account_info(),
      };
      let cpi_program = ctx.accounts.token_program.to_account_info();
      let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer_seeds);
      token::transfer(cpi_ctx, asset_amount)?;



      simple_pool.volatile_share_borrowed=simple_pool.volatile_share_borrowed.checked_add(requested_volatile_share).unwrap();
      //simple_pool.volatile_deposited=simple_pool.volatile_deposited.checked_add(asset_amount).unwrap();
      user_account.user_volatile_share_borrowed=user_account.user_volatile_share_borrowed.checked_add(requested_volatile_share).unwrap();
      }

    }
  
    //simple_pool.accrue_yield(Clock::get()?.unix_timestamp);
    Ok(())
}


#[derive(Accounts)]
pub struct SUserBorrows<'info> {
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


    #[account(mut,
        seeds = [b"vault",simple_pool.stable_mint.key().as_ref(),simple_pool.key().as_ref()],
        bump,
        
      )]
    pub stable_vault: Box<Account<'info, TokenAccount>>,


    #[account(mut,
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

    #[account(mut,
      token::mint = simple_pool.stable_mint,
      token::authority = user_signer)]
    pub user_stable_vault: Box<Account<'info, TokenAccount>>,

    #[account(mut)]
    pub user_volatile_vault: Box<Account<'info, TokenAccount>>,



    #[account(
      constraint=pyth_feed.key()==simple_pool.pyth_feed.key()
    )]
    pub pyth_feed: Account<'info, PriceUpdateV2>,


    // Programs and Sysvars
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub rent: Sysvar<'info, Rent>,
}

