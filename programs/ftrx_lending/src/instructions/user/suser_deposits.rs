//libraries
use anchor_lang::prelude::*;
use anchor_spl::token::{self, Burn, Mint, Token, TokenAccount, Transfer};
//local imports

use crate::states::{simple_pool::SimplePool};
use crate::states::{simple_ua::SimpleUa};
use crate::errors::ErrorCode;

pub fn handle(
    ctx: Context<SUserDeposits>,
    asset_index: u8,
    asset_amount:u64,
)->Result<()>{
    let mut simple_pool = &mut ctx.accounts.simple_pool;
    let mut user_account = &mut ctx.accounts.user_state;
    let SHARE_VALUE_MULTIPLIER:u64=1_000_000_000_000;
    if (asset_index==0){
        if (asset_amount<10_000) {return err!(ErrorCode::AmountTooLow); }
        if (asset_amount>1_000_000_000) {return err!(ErrorCode::AmountTooBig); }
        
        let intermediatry_amount_volatile=asset_amount;
        let adapted_transfer_amount=intermediatry_amount_volatile;
        
        let cpi_accounts = Transfer {
            from: ctx.accounts.user_stable_vault.to_account_info(),
            to: ctx.accounts.stable_vault.to_account_info(),
            authority: ctx.accounts.user_signer.to_account_info(),
        };
        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
        token::transfer(cpi_ctx, adapted_transfer_amount)?;
        
        let share_deposited=adapted_transfer_amount.checked_mul(SHARE_VALUE_MULTIPLIER).unwrap().checked_div(simple_pool.stable_share_asset_value).unwrap();
        simple_pool.stable_share_deposited=simple_pool.stable_share_deposited.checked_add(share_deposited).unwrap();
        //simple_pool.stable_deposited=simple_pool.stable_deposited.checked_add(asset_amount).unwrap();
        user_account.user_stable_share_deposited=user_account.user_stable_share_deposited.checked_add(share_deposited).unwrap();
        msg!["After stable deposit volatile_share_deposited {}, simple_pool.stable_share_deposited {}",simple_pool.volatile_share_deposited,simple_pool.stable_share_deposited];
        msg!["After sol deposit volatile_share_deposited {}, simple_pool.stable_share_deposited {}",user_account.user_volatile_share_deposited,user_account.user_stable_share_deposited];
    
      }else{

        if (asset_amount<10_000) {return err!(ErrorCode::AmountTooLow); }
        if (asset_amount>1_000_000_000) {return err!(ErrorCode::AmountTooBig); }
        
        
        let intermediatry_amount_volatile=asset_amount.checked_div(1_000).unwrap();
        let adapted_transfer_amount=intermediatry_amount_volatile.checked_mul(1_000).unwrap();


        // transfer sol to token account
        let cpi_context = CpiContext::new(
          ctx.accounts.system_program.to_account_info(),
          anchor_lang::system_program::Transfer {
              from: ctx.accounts.user_signer.to_account_info(),
              to: ctx.accounts.volatile_vault.to_account_info(),
          });
          anchor_lang::system_program::transfer(cpi_context, adapted_transfer_amount)?;
  
          // Sync the native token to reflect the new SOL balance as wSOL
          let cpi_accounts = token::SyncNative {
              account: ctx.accounts.volatile_vault.to_account_info(),
          };
          let cpi_program = ctx.accounts.token_program.to_account_info();
          let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
          token::sync_native(cpi_ctx)?;

          let intermediatry_amount_volatile_128=adapted_transfer_amount as u128;
          let volatile_share_deposited_by_user=intermediatry_amount_volatile_128.checked_mul(SHARE_VALUE_MULTIPLIER as u128).unwrap().checked_div(simple_pool.volatile_share_asset_value as u128).unwrap().checked_div(1_000).unwrap() as u64;
          simple_pool.volatile_share_deposited=simple_pool.volatile_share_deposited.checked_add(volatile_share_deposited_by_user).unwrap();
          //simple_pool.volatile_deposited=simple_pool.volatile_deposited.checked_add(asset_amount).unwrap();
          user_account.user_volatile_share_deposited=user_account.user_volatile_share_deposited.checked_add(volatile_share_deposited_by_user).unwrap();
          msg!["After sol deposit volatile_share_deposited global pool {}, simple_pool.stable_share_deposited {}",simple_pool.volatile_share_deposited,simple_pool.stable_share_deposited];
          msg!["After sol deposit volatile_share_deposited {}, simple_pool.stable_share_deposited {}",user_account.user_volatile_share_deposited,user_account.user_stable_share_deposited];
    
    }

  Ok(())
}


#[derive(Accounts)]
pub struct SUserDeposits<'info> {
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


    #[account(mut,
      token::mint = simple_pool.stable_mint,
      token::authority = user_signer)]
    pub user_stable_vault: Box<Account<'info, TokenAccount>>,



    #[account(
      constraint = stable_mint.key() == simple_pool.stable_mint.key())]
    pub stable_mint: Box<Account<'info, Mint>>,
    #[account(
      constraint = volatile_mint.key() == simple_pool.volatile_mint.key())]
    pub volatile_mint: Box<Account<'info, Mint>>,

    // Programs and Sysvars
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub rent: Sysvar<'info, Rent>,
}

