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
    ctx: Context<SUserRedeems>,
    asset_index: u8,
    share_amount:u64,
)->Result<()>{

  let simple_pool = &mut ctx.accounts.simple_pool;
  let mut user_account = &mut ctx.accounts.user_state;
  let SHARE_VALUE_MULTIPLIER:u64=1_000_000_000_000;
  if asset_index==0{
    if (share_amount<10_000) {return err!(ErrorCode::AmountTooLow); }
    if (share_amount>1_000_000_000) {return err!(ErrorCode::AmountTooBig); }
    

    if (user_account.user_stable_share_borrowed==0) {return err!(ErrorCode::NoBorrowWantsToRedeem); }

    let amount_stable_share_returned=share_amount;
    msg!["Redemption : amount stable share attempted returned {}/1000000 ",amount_stable_share_returned];
    
    let asset_amount_to_transfer=share_amount.checked_mul(simple_pool.stable_share_liabi_value).unwrap().checked_div(SHARE_VALUE_MULTIPLIER).unwrap();

    let cpi_accounts = Transfer {
      from: ctx.accounts.user_stable_vault.to_account_info(),
      to: ctx.accounts.stable_vault.to_account_info(),
      authority: ctx.accounts.user_signer.to_account_info(),
    };
    let cpi_program = ctx.accounts.token_program.to_account_info();
    let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
    token::transfer(cpi_ctx, asset_amount_to_transfer)?;
    


    simple_pool.stable_share_borrowed=simple_pool.stable_share_borrowed.checked_sub(amount_stable_share_returned).unwrap();
    //simple_pool.volatile_deposited=simple_pool.volatile_deposited.checked_add(asset_amount).unwrap();
    msg!["Redemption : borrowed stable share before {}/1000000 {}/1000000",user_account.user_stable_share_borrowed,amount_stable_share_returned];
    user_account.user_stable_share_borrowed=user_account.user_stable_share_borrowed.checked_sub(amount_stable_share_returned).unwrap();


    

  }else{
    if (share_amount<10_000) {return err!(ErrorCode::AmountTooLow); }
    if (share_amount>1_000_000_000) {return err!(ErrorCode::AmountTooBig); }
    
    if (user_account.user_volatile_share_deposited>0) {return err!(ErrorCode::NoBorrowIfDeposited); }

    let amount_volatile_share_returned=share_amount;
    let amount_volatile_share_returned_128=share_amount as u128;
    let amount_volatile_token_to_transfer:u64=amount_volatile_share_returned_128.checked_mul(1000).unwrap().checked_mul(simple_pool.volatile_share_liabi_value as u128).unwrap().checked_div(SHARE_VALUE_MULTIPLIER as u128).unwrap() as u64;

    
    // transfer sol to token account
    let cpi_context = CpiContext::new(
      ctx.accounts.system_program.to_account_info(),
      anchor_lang::system_program::Transfer {
          from: ctx.accounts.user_signer.to_account_info(),
          to: ctx.accounts.volatile_vault.to_account_info(),
      });
      anchor_lang::system_program::transfer(cpi_context, amount_volatile_token_to_transfer)?;

      // Sync the native token to reflect the new SOL balance as wSOL
      let cpi_accounts = token::SyncNative {
          account: ctx.accounts.volatile_vault.to_account_info(),
      };
      let cpi_program = ctx.accounts.token_program.to_account_info();
      let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
      token::sync_native(cpi_ctx)?;

    simple_pool.volatile_share_borrowed=simple_pool.volatile_share_borrowed.checked_sub(amount_volatile_share_returned).unwrap();
    //simple_pool.volatile_deposited=simple_pool.volatile_deposited.checked_add(asset_amount).unwrap();
    user_account.user_volatile_share_borrowed=user_account.user_volatile_share_borrowed.checked_sub(amount_volatile_share_returned).unwrap();


  }

    Ok(())
}


#[derive(Accounts)]
pub struct SUserRedeems<'info> {
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

