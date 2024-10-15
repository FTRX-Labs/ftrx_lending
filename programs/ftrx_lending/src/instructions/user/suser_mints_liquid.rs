//libraries
use anchor_lang::prelude::*;
use pyth_solana_receiver_sdk::price_update::{PriceUpdateV2,get_feed_id_from_hex};
use std::mem::size_of;

use anchor_spl::token::{self, Mint, MintTo, Token, TokenAccount, Transfer};
//local imports

use crate::states::{simple_pool::SimplePool};
use crate::states::{simple_ua::SimpleUa};
use crate::errors::ErrorCode;

pub fn handle(
    ctx: Context<SUserMintsLiquid>,
    asset_index:u8,
    asset_amount: u64,
)->Result<()>{
  let SHARE_VALUE_MULTIPLIER:u64=1_000_000_000_000;
  let price_update = &mut ctx.accounts.pyth_feed;
  let simple_pool = &ctx.accounts.simple_pool;
  let mut user_account = &mut ctx.accounts.user_state;
    // get_price_no_older_than will fail if the price update is more than 30 seconds old
    let maximum_age: u64 = 300;
    // get_price_no_older_than will fail if the price update is for a different price feed.
    // This string is the id of the BTC/USD feed. See https://pyth.network/developers/price-feed-ids for all available IDs.
    let feed_id: [u8; 32] = get_feed_id_from_hex("ef0d8b6fda2ceba41da15d4095d1da392a0d2f8ed0c6c7bc0f4cfac8c280b56d")?;
    msg!("Is the price found?");
    let price = price_update.get_price_no_older_than(&Clock::get()?, maximum_age, &feed_id)?;
    
    msg!("The price is ({} ± {}) * 10^{}", price.price, price.conf, price.exponent);
    
    let base: f64 = 10.0;
    let multiplier= base.powf(-price.exponent as f64) as u64;
    
   
    if asset_index==0{

        let mut free_margin_in_asset_share_stable=user_account.get_max_new_mint_liquid_stable(&simple_pool,price.price,multiplier).unwrap();
        if (asset_amount<free_margin_in_asset_share_stable){
            //Mint the liquid stable
          
            let signer_seeds: &[&[&[u8]]] = &[&[
              simple_pool.stable_mint.as_ref(),
              simple_pool.volatile_mint.as_ref(),
              simple_pool.pool_admin.as_ref(),
              &[simple_pool.pool_bump],
          ]];

            let cpi_accounts = MintTo {
              mint: ctx.accounts.liquid_stable_mint.to_account_info(),
              to: ctx.accounts.user_liquid_stable_uta.to_account_info(),
              authority: ctx.accounts.simple_pool.to_account_info(),
            };
            let cpi_program = ctx.accounts.token_program.to_account_info();
            let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer_seeds);
            token::mint_to(cpi_ctx, asset_amount)?;

            let locked_share=asset_amount.checked_mul(SHARE_VALUE_MULTIPLIER).unwrap().checked_div(simple_pool.stable_share_asset_value).unwrap();
            user_account.stable_asset_share_locked=user_account.stable_asset_share_locked.checked_add(locked_share).unwrap();
            user_account.liquid_stable_minted=user_account.liquid_stable_minted.checked_add(asset_amount).unwrap();
            

          
        }
    }else{
        let mut free_margin_in_asset_share_volatile=user_account.get_max_new_mint_liquid_volatile(&simple_pool,price.price,multiplier).unwrap();
        free_margin_in_asset_share_volatile=free_margin_in_asset_share_volatile.checked_mul(1_000).unwrap();
        if (asset_amount<free_margin_in_asset_share_volatile){
            //Mint the liquid volatile
           
            let signer_seeds: &[&[&[u8]]] = &[&[
              simple_pool.stable_mint.as_ref(),
              simple_pool.volatile_mint.as_ref(),
              simple_pool.pool_admin.as_ref(),
              &[simple_pool.pool_bump],
          ]];

            let cpi_accounts = MintTo {
              mint: ctx.accounts.liquid_volatile_mint.to_account_info(),
              to: ctx.accounts.user_liquid_volatile_uta.to_account_info(),
              authority: ctx.accounts.simple_pool.to_account_info(),
            };
            let cpi_program = ctx.accounts.token_program.to_account_info();
            let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer_seeds);
            token::mint_to(cpi_ctx, asset_amount)?;

            
            let adapted_asset_amount=asset_amount.checked_div(1_000).unwrap();
            let locked_share=adapted_asset_amount.checked_mul(SHARE_VALUE_MULTIPLIER).unwrap().checked_div(simple_pool.volatile_share_asset_value).unwrap();
            user_account.volatile_asset_share_locked=user_account.volatile_asset_share_locked.checked_add(locked_share).unwrap();
            user_account.liquid_volatile_minted=user_account.liquid_volatile_minted.checked_add(asset_amount).unwrap();

        }
    }


    Ok(())
}



#[derive(Accounts)]
pub struct SUserMintsLiquid<'info> {
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

