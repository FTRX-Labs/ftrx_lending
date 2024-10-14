//libraries
use anchor_lang::prelude::*;
use pyth_solana_receiver_sdk::price_update::{PriceUpdateV2};
use std::mem::size_of;
use anchor_spl::token::{Mint, Token, TokenAccount};
//local imports

use crate::states::{simple_pool::SimplePool};
use crate::states::{simple_ua::SimpleUa};

pub fn handle(
    ctx: Context<SUserCreatesUa>,
    bump: u8,
)->Result<()>{

    let user_state = &mut ctx.accounts.user_state;
    user_state.ua_bump=bump;
    user_state.pool_pk=ctx.accounts.simple_pool.key();
    user_state.ua_authority=ctx.accounts.user_authority.key();
    user_state.user_stable_share_deposited=0;
    user_state.user_volatile_share_deposited=0;
    user_state.user_stable_share_borrowed=0;
    user_state.user_volatile_share_borrowed=0;


    
    user_state.ua_bump=bump;

    Ok(())
}


#[derive(Accounts)]
pub struct SUserCreatesUa<'info> {
    // Super User
    #[account(mut)]
    pub user_authority: Signer<'info>,
    #[account(init,
        seeds = [simple_pool.key().as_ref(), user_authority.key().as_ref()],
        bump,
        payer = user_authority,
        space = 8 + size_of::<SimpleUa>()
        )]
    pub user_state: Box<Account<'info, SimpleUa>>,


    pub simple_pool: Box<Account<'info, SimplePool>>,


    // Programs and Sysvars
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub rent: Sysvar<'info, Rent>,
}

