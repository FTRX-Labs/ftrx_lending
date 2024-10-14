// libraries
use anchor_lang::prelude::*;

//local imports
pub mod instructions;
pub mod states;
pub mod errors;

// crates
use crate::instructions::*;

declare_id!("CPEZgD3UunRhzzzrowtt4zHRgngYjumBCqhFqEG9EwNx");

#[program]
pub mod ftrx_lending {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        Ok(())
    }

    pub fn admin_creates_sp(ctx: Context<AdminCreatesSp>,
                            bump: u8,
                        target_utilization_in: u64,
                        protocol_fee_in: u64,
                        insurance_fund_fee_in:u64,
                        new_borrow_max_ltv:u64,
                        liquidation_ltv:u64,
                        stable_decimal:u8,
                        volatile_decimal:u8,
                    ) -> Result<()> {
        admin_creates_sp::handle(ctx,bump,target_utilization_in,protocol_fee_in,insurance_fund_fee_in,new_borrow_max_ltv,liquidation_ltv,stable_decimal,volatile_decimal);
        Ok(())
    }

    pub fn suser_creates_ua(ctx: Context<SUserCreatesUa>,
        bump: u8,
        )-> Result<()> {
    suser_creates_ua::handle(ctx,bump);
    Ok(())
    }

    pub fn suser_deposits(ctx: Context<SUserDeposits>,
        asset_index: u8,
        asset_amount:u64,)-> Result<()> {
    suser_deposits::handle(ctx,asset_index,asset_amount);
    Ok(())
    }


    pub fn suser_withdraws(ctx: Context<SUserWithdraws>,
        asset_index: u8,
        asset_amount:u64,)-> Result<()> {
    suser_withdraws::handle(ctx,asset_index,asset_amount);
    Ok(())
    }

    pub fn suser_borrows(ctx: Context<SUserBorrows>,
        asset_index: u8,
        asset_amount:u64,)-> Result<()> {
            suser_borrows::handle(ctx,asset_index,asset_amount);
    Ok(())
    }

    pub fn suser_redeems(ctx: Context<SUserRedeems>,
        asset_index: u8,
        asset_amount:u64,)-> Result<()> {
            suser_redeems::handle(ctx,asset_index,asset_amount);
    Ok(())
    }

    pub fn suser_liquidates(ctx: Context<SUserLiquidates>,
        asset_index: u8,
        asset_amount:u64,)-> Result<()> {
            suser_liquidates::handle(ctx,asset_index,asset_amount);
    Ok(())
    }

    pub fn admin_liquidates_sp(ctx: Context<AdminLiquidatesSP>,
        asset_index: u8,
        asset_amount:u64,)-> Result<()> {
            admin_liquidates_sp::handle(ctx,asset_index,asset_amount);
    Ok(())
    }



}

#[derive(Accounts)]
pub struct Initialize {}
