//libraries
use anchor_lang::prelude::*;
use pyth_solana_receiver_sdk::price_update::{PriceUpdateV2};
use std::mem::size_of;
use anchor_spl::token::{Mint, Token, TokenAccount};
//local imports

use crate::states::{simple_pool::SimplePool};
use crate::states::{simple_ua::SimpleUa};


pub fn handle(
    ctx: Context<AdminLiquidatesSP>,
    asset_index: u8,
    asset_amount:u64,
)->Result<()>{

    Ok(())
}


#[derive(Accounts)]
pub struct AdminLiquidatesSP<'info> {
    // Super User
    #[account(
      seeds = [stable_mint.key().as_ref(),volatile_mint.key().as_ref(), simple_pool.pool_admin.key().as_ref()],
      bump,
    )]
    pub simple_pool: Box<Account<'info, SimplePool>>,


    #[account(
      seeds = [simple_pool.key().as_ref(), user_state.ua_authority.key().as_ref()],
      bump,
      constraint=user_state.ua_authority.key()==user_signer.key()
      )]
    pub user_state: Box<Account<'info, SimpleUa>>,


    #[account(
      seeds = [simple_pool.key().as_ref(), user_to_liquidate.ua_authority.key().as_ref()],
      bump,
      
      )]
    pub user_to_liquidate: Box<Account<'info, SimpleUa>>,


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

