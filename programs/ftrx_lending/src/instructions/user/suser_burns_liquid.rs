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
    ctx: Context<SUserBurnsLiquid>,
    asset_index:u8,
    asset_amount: u64,
)->Result<()>{
    let simple_pool = &mut ctx.accounts.simple_pool;
    let mut user_account = &mut ctx.accounts.user_state;

    if asset_index==0{

        let cpi_accounts = Burn {
            mint: ctx.accounts.simple_pool.to_account_info(),
            from: ctx.accounts.user_liquid_stable_uta.to_account_info(),
            authority: ctx.accounts.user_signer.to_account_info(),
        };
        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
        token::burn(cpi_ctx, asset_amount)?;

        let mut simple_pool = &mut ctx.accounts.simple_pool;
        user_account.liquid_stable_minted=user_account.liquid_stable_minted.checked_sub(asset_amount).unwrap();
        let pct_of_stable_returned=asset_amount.checked_mul(1_000_000).unwrap().checked_div(user_account.liquid_stable_minted).unwrap();
        let amount_of_share_to_unlock=user_account.stable_asset_share_locked.checked_mul(pct_of_stable_returned).unwrap().checked_div(1_000_000).unwrap();
        user_account.stable_asset_share_locked=user_account.stable_asset_share_locked.checked_sub(amount_of_share_to_unlock).unwrap();


    }else{
 
        let cpi_accounts = Burn {
            mint: ctx.accounts.simple_pool.to_account_info(),
            from: ctx.accounts.user_liquid_volatile_uta.to_account_info(),
            authority: ctx.accounts.user_signer.to_account_info(),
        };
        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
        token::burn(cpi_ctx, asset_amount)?;

        let adapted_asset_amount=asset_amount.checked_div(1_000).unwrap();
        user_account.liquid_volatile_minted=user_account.liquid_volatile_minted.checked_sub(asset_amount).unwrap();
        let pct_of_volatile_returned=asset_amount.checked_mul(1_000_000).unwrap().checked_div(user_account.liquid_volatile_minted).unwrap();
        let amount_of_share_to_unlock=user_account.volatile_asset_share_locked.checked_mul(pct_of_volatile_returned).unwrap().checked_div(1_000_000).unwrap();
        user_account.volatile_asset_share_locked=user_account.volatile_asset_share_locked.checked_sub(amount_of_share_to_unlock).unwrap();

    }


    Ok(())
}



#[derive(Accounts)]
pub struct SUserBurnsLiquid<'info> {
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

        seeds = [b"liquid",stable_mint.key().as_ref(),simple_pool.key().as_ref()],
        bump,
      )]
      pub liquid_stable_mint: Box<Account<'info, Mint>>,
  
  
      #[account(mut,
  
        seeds = [b"liquid",volatile_mint.key().as_ref(),simple_pool.key().as_ref()],
        bump,
      )]
      pub liquid_volatile_mint: Box<Account<'info, Mint>>,
  

      

    #[account(mut)]
    pub user_liquid_stable_uta: Box<Account<'info, TokenAccount>>,

    #[account(mut)]
    pub user_liquid_volatile_uta: Box<Account<'info, TokenAccount>>,



    #[account(
      constraint=pyth_feed.key()==simple_pool.pyth_feed.key()
    )]
    pub pyth_feed: Account<'info, PriceUpdateV2>,


    // Programs and Sysvars
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub rent: Sysvar<'info, Rent>,
}

