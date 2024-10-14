//libraries
use anchor_lang::prelude::*;
use pyth_solana_receiver_sdk::price_update::{PriceUpdateV2};
use std::mem::size_of;
use anchor_spl::token::{self, Burn, Mint, Token, TokenAccount, Transfer};
//local imports

use crate::states::{simple_pool::SimplePool};
use crate::states::{simple_ua::SimpleUa};
use crate::errors::ErrorCode;

pub fn handle(
    ctx: Context<SUserWithdraws>,
    asset_index: u8,
    asset_amount:u64,
)->Result<()>{

    let mut simple_pool = &mut ctx.accounts.simple_pool;
    let mut user_account = &mut ctx.accounts.user_state;
    let SHARE_VALUE_MULTIPLIER:u64=1_000_000_000_000;
    if (asset_index==0){

        if (asset_amount<10_000) {return err!(ErrorCode::AmountTooLow); }
        if (asset_amount>1_000_000_000) {return err!(ErrorCode::AmountTooBig); }
        
        
        let simple_pool = &mut ctx.accounts.simple_pool;
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

      let share_amount_to_redeeem=asset_amount.checked_mul(simple_pool.stable_share_asset_value).unwrap().checked_div(SHARE_VALUE_MULTIPLIER).unwrap();
  
      simple_pool.stable_share_deposited=simple_pool.stable_share_deposited.checked_sub(share_amount_to_redeeem).unwrap();
      user_account.user_stable_share_deposited=user_account.user_stable_share_deposited.checked_sub(share_amount_to_redeeem).unwrap();
    
    }else{

      if (asset_amount<10_000) {return err!(ErrorCode::AmountTooLow); }
      if (asset_amount>1_000_000_000) {return err!(ErrorCode::AmountTooBig); }
      
      
      let simple_pool = &mut ctx.accounts.simple_pool;
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

      let share_amount_to_redeeem=asset_amount.checked_mul(simple_pool.volatile_share_asset_value).unwrap().checked_div(SHARE_VALUE_MULTIPLIER).unwrap();
  
      simple_pool.volatile_share_deposited=simple_pool.volatile_share_deposited.checked_sub(share_amount_to_redeeem).unwrap();
      user_account.user_volatile_share_deposited=user_account.user_volatile_share_deposited.checked_sub(share_amount_to_redeeem).unwrap();


    }
    Ok(())
}


#[derive(Accounts)]
pub struct SUserWithdraws<'info> {
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


    // Programs and Sysvars
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub rent: Sysvar<'info, Rent>,
}

