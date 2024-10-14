//libraries
use anchor_lang::prelude::*;
use pyth_solana_receiver_sdk::price_update::{PriceUpdateV2};
use std::mem::size_of;
use anchor_spl::token::{Mint, Token, TokenAccount};
//local imports

use crate::states::{simple_pool::SimplePool};

pub fn handle(
    ctx: Context<AdminCreatesSp>,
    bump: u8,
    target_utilization_in: u64,
    protocol_fee_in: u64,
    insurance_fund_fee_in:u64,
    new_borrow_max_ltv:u64,
    liquidation_ltv:u64,
    stable_decimal:u8,
    volatile_decimal:u8,

)->Result<()>{

    let simple_pool_element = &mut ctx.accounts.simple_pool;
    
    
    simple_pool_element.pool_bump=bump;
    simple_pool_element.pool_admin=ctx.accounts.pool_admin.key();
    
    simple_pool_element.stable_mint=ctx.accounts.stable_mint.key();
    simple_pool_element.volatile_mint=ctx.accounts.volatile_mint.key();


    simple_pool_element.volatile_share_mint=ctx.accounts.volatile_share_mint.key();
    simple_pool_element.stable_share_mint=ctx.accounts.stable_share_mint.key();


    simple_pool_element.stable_vault=ctx.accounts.stable_vault.key();
    simple_pool_element.volatile_vault=ctx.accounts.volatile_vault.key();

    simple_pool_element.pyth_feed=ctx.accounts.pyth_feed.key();
 

    simple_pool_element.stable_share_deposited=0;
    simple_pool_element.volatile_share_deposited=0;

    simple_pool_element.stable_share_asset_value=  1_000_000_000_000;
    simple_pool_element.stable_share_liabi_value=  1_000_000_000_000;
    simple_pool_element.volatile_share_asset_value=1_000_000_000_000;
    simple_pool_element.volatile_share_liabi_value=1_000_000_000_000;


    simple_pool_element.target_utilization=target_utilization_in;
    simple_pool_element.new_borrow_max_ltv=new_borrow_max_ltv;
    simple_pool_element.liquidation_ltv=liquidation_ltv;

    simple_pool_element.bd_discount_stable=0;
    simple_pool_element.bd_discount_volatile=0;

    simple_pool_element.protocol_fee=protocol_fee_in;
    simple_pool_element.insurance_fund_fee=insurance_fund_fee_in;
    
    simple_pool_element.last_update=Clock::get()?.unix_timestamp;
    simple_pool_element.last_price=0;

    Ok(())
}


#[derive(Accounts)]
pub struct AdminCreatesSp<'info> {
    // Super User
    #[account(init,
      seeds = [stable_mint.key().as_ref(),volatile_mint.key().as_ref(), pool_admin.key().as_ref()],
      bump,
      payer = pool_admin,
      space = 8 + size_of::<SimplePool>()
    )]
    pub simple_pool: Box<Account<'info, SimplePool>>,

    #[account(mut)]
    pub pool_admin: Signer<'info>,


    #[account(init,
        mint::decimals = 6,
        mint::authority = simple_pool,
        seeds = [b"share",volatile_mint.key().as_ref(),simple_pool.key().as_ref()],
        bump,
        payer = pool_admin
    )]
    pub volatile_share_mint: Box<Account<'info, Mint>>,


    #[account(init,
      mint::decimals = 6,
      mint::authority = simple_pool,
      seeds = [b"share",stable_mint.key().as_ref(),simple_pool.key().as_ref()],
      bump,
      payer = pool_admin
    )]
    pub stable_share_mint: Box<Account<'info, Mint>>,


    #[account(init,
        token::mint = stable_mint,
        token::authority = simple_pool,
        seeds = [b"vault",stable_mint.key().as_ref(),simple_pool.key().as_ref()],
        bump,
        payer = pool_admin
      )]
    pub stable_vault: Box<Account<'info, TokenAccount>>,


    #[account(init,
      token::mint = volatile_mint,
      token::authority = simple_pool,
      seeds = [b"vault",volatile_mint.key().as_ref(),simple_pool.key().as_ref()],
      bump,
      payer = pool_admin
    )]
    pub volatile_vault: Box<Account<'info, TokenAccount>>,



    pub stable_mint: Box<Account<'info, Mint>>,
    pub volatile_mint: Box<Account<'info, Mint>>,


    pub pyth_feed: Account<'info, PriceUpdateV2>,


    // Programs and Sysvars
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub rent: Sysvar<'info, Rent>,
}

