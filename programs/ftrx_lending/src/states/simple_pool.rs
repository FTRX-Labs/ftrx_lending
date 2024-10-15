use anchor_lang::prelude::*;


#[account]
#[derive(Default)]
pub struct SimplePool{
    pub pool_bump:u8,
    pub pool_admin:Pubkey,
    pub pool_type:u8,

    pub pyth_feed:Pubkey,
    pub switchboard_feed:Pubkey,


    pub mint: Pubkey,
    

    pub stable_vault:Pubkey,
    pub volatile_vault:Pubkey,

    pub ct_stable_vault:Pubkey,
    pub ct_volatile_vault:Pubkey,

    pub stable_decimal:u8,
    pub volatile_decimal:u8,

    pub stable_mint:Pubkey,
    pub volatile_mint:Pubkey,

    pub stable_share_mint:Pubkey,
    pub volatile_share_mint:Pubkey,

    pub stable_share_asset_value:u64,
    pub stable_share_liabi_value:u64,
    pub volatile_share_asset_value:u64,
    pub volatile_share_liabi_value:u64,

    pub volatile_deposited:u64,
    pub stable_deposited:u64,

    pub stable_share_deposited:u64,
    pub volatile_share_deposited:u64,

    pub stable_share_borrowed:u64,
    pub volatile_share_borrowed:u64,


    pub target_utilization:u64,

    pub bd_discount_stable:u64,
    pub bd_discount_volatile:u64,

    pub protocol_fee:u64,
    pub insurance_fund_fee:u64,
    pub liquidation_penalty:u64,

    pub new_borrow_max_ltv:u64,
    pub liquidation_ltv:u64,
    pub liquidation_max_ltv:u64,

    pub last_update:i64,
    pub last_price:i64,
}


impl SimplePool{


    pub fn calculate_base_rate_apr(
        &mut self,
        utilization_rate: u128,
        )
        -> Option<(u128)>{
            let BASE_MULTIPLIER:u128=1_000_000_000_000;
            let top_yield_one:u128=60;
            let long_term_yield:u128=top_yield_one.checked_mul(BASE_MULTIPLIER).unwrap().checked_div(100).unwrap();

            Some(long_term_yield)
    }

    pub fn accrue_yield(
        &mut self,
        current_timestamp: i64,
        )
        -> Result<()>{
            self.accrue_yield_stable(current_timestamp);
            self.accrue_yield_volatile(current_timestamp);
            self.last_update=current_timestamp;
            Ok(())
    }

    pub fn accrue_yield_volatile(
        &mut self,
        current_timestamp: i64,
        )
        -> Result<()>{
            let BASE_MULTIPLIER:u128=1_000_000_000_000;

            //30 is 30%
            let long_term_yield:u128=60;

            //Time gap
            let time_delta=current_timestamp.checked_sub(self.last_update).unwrap();
            if time_delta == 0 {
                return Ok(());
            }
            //Amounts with 6 decimals
            let v_share_deposited:u128=self.volatile_share_deposited as u128;
            let v_share_borrowed:u128 =self.volatile_share_borrowed  as u128;
            let v_share_asset_value:u128    =self.volatile_share_asset_value     as u128;
            let v_share_liabi_value:u128    =self.volatile_share_liabi_value     as u128;
            //Amounts with 6 decimals
            let volatile_deposited_amount=v_share_deposited.checked_mul(v_share_asset_value).unwrap();
            let volatile_borrowed_amount= v_share_borrowed.checked_mul(v_share_liabi_value).unwrap();
            msg!("Log accruing yield : volatile_deposited_amount {},volatile_borrowed_amount {}",volatile_deposited_amount,volatile_borrowed_amount);
            if (volatile_borrowed_amount==0 || volatile_deposited_amount==0){
                msg!("Log accruing yield : no borrowed or deposited");
                return Ok(());
            }

            
            let utilization_rate:u128=volatile_borrowed_amount.checked_mul(BASE_MULTIPLIER).unwrap().checked_div(volatile_deposited_amount).unwrap();
            msg!("Log utilization_rate : {}/1000000000000",utilization_rate);
            
            //To divide by 1M
            let base_rate=self.calculate_base_rate_apr(utilization_rate).unwrap();
            msg!("Log base rate : {}/1000000000000",base_rate);
            
            let fixed_apr_fee=self.protocol_fee.checked_add(self.insurance_fund_fee).unwrap();
            msg!("Log fixed_apr_fee : {}/1000000000000",fixed_apr_fee);
            
            //Rate received by lenders = depositors of assets
            let lender_rate_apr=base_rate.checked_mul(utilization_rate).unwrap().checked_div(BASE_MULTIPLIER).unwrap();
            let impact_lender_rate=lender_rate_apr.checked_mul(time_delta as u128).unwrap().checked_div(31536000).unwrap();
            let final_lender_rate=impact_lender_rate.checked_add(BASE_MULTIPLIER).unwrap();
            msg!("Log lender rate : {}/1000000000000",final_lender_rate);
            msg!("Log lender_rate APR : {}/1000000000000",lender_rate_apr);
            
            let borrowing_rate_apr=base_rate.checked_add(fixed_apr_fee as u128).unwrap();
            let impact_borrowing_rate=borrowing_rate_apr.checked_mul(time_delta as u128).unwrap().checked_div(31536000).unwrap();
            let final_borrowing_rate=impact_borrowing_rate.checked_add(BASE_MULTIPLIER).unwrap();
            msg!("Log borrowing_rate_apr rate : {}/1000000000000",final_borrowing_rate);
            
            let new_volatile_share_asset_value=v_share_asset_value.checked_mul(final_lender_rate).unwrap().checked_div(BASE_MULTIPLIER).unwrap();
            let new_volatile_share_liabi_value=v_share_liabi_value.checked_mul(final_borrowing_rate).unwrap().checked_div(BASE_MULTIPLIER).unwrap();
            
            msg!("Log old asset values : asset {}/1000000000000 liability {}/1000000000000",v_share_asset_value,v_share_liabi_value);
            
            msg!("Log new asset values : asset {}/1000000000000 liability {}/1000000000000",new_volatile_share_asset_value,new_volatile_share_liabi_value);
            
            self.volatile_share_asset_value=new_volatile_share_asset_value as u64;
            self.volatile_share_liabi_value=new_volatile_share_liabi_value as u64;
            
            Ok(())
    }
    pub fn accrue_yield_stable(
        &mut self,
        current_timestamp: i64,
        )
        -> Result<()>{
            let BASE_MULTIPLIER:u128=1_000_000_000_000;

            //30 is 30%
            let long_term_yield:u128=60;

            //Time gap
            let time_delta=current_timestamp.checked_sub(self.last_update).unwrap();
            if time_delta == 0 {
                return Ok(());
            }
            //Amounts with 6 decimals
            let v_share_deposited:u128=self.stable_share_deposited as u128;
            let v_share_borrowed:u128 =self.stable_share_borrowed  as u128;
            let v_share_asset_value:u128    =self.stable_share_asset_value     as u128;
            let v_share_liabi_value:u128    =self.stable_share_liabi_value     as u128;
            //Amounts with 6 decimals
            let stable_deposited_amount=v_share_deposited.checked_mul(v_share_asset_value).unwrap();
            let stable_borrowed_amount= v_share_borrowed.checked_mul(v_share_liabi_value).unwrap();
            msg!("Log accruing yield : stable_deposited_amount {},stable_borrowed_amount {}",stable_deposited_amount,stable_borrowed_amount);
            if (stable_borrowed_amount==0 || stable_deposited_amount==0){
                msg!("Log accruing yield : no borrowed or deposited");
                return Ok(());
            }

            
            let utilization_rate:u128=stable_borrowed_amount.checked_mul(BASE_MULTIPLIER).unwrap().checked_div(stable_deposited_amount).unwrap();
            msg!("Log utilization_rate : {}/1000000000000",utilization_rate);
            
            //To divide by 1M
            let base_rate=self.calculate_base_rate_apr(utilization_rate).unwrap();
            msg!("Log base rate : {}/1000000000000",base_rate);
            
            let fixed_apr_fee=self.protocol_fee.checked_add(self.insurance_fund_fee).unwrap();
            msg!("Log fixed_apr_fee : {}/1000000000000",fixed_apr_fee);
            
            //Rate received by lenders = depositors of assets
            let lender_rate_apr=base_rate.checked_mul(utilization_rate).unwrap().checked_div(BASE_MULTIPLIER).unwrap();
            let impact_lender_rate=lender_rate_apr.checked_mul(time_delta as u128).unwrap().checked_div(31536000).unwrap();
            let final_lender_rate=impact_lender_rate.checked_add(BASE_MULTIPLIER).unwrap();
            msg!("Log lender rate : {}/1000000000000",final_lender_rate);
            msg!("Log lender_rate APR : {}/1000000000000",lender_rate_apr);
            
            let borrowing_rate_apr=base_rate.checked_add(fixed_apr_fee as u128).unwrap();
            let impact_borrowing_rate=borrowing_rate_apr.checked_mul(time_delta as u128).unwrap().checked_div(31536000).unwrap();
            let final_borrowing_rate=impact_borrowing_rate.checked_add(BASE_MULTIPLIER).unwrap();
            msg!("Log borrowing_rate_apr rate : {}/1000000000000",final_borrowing_rate);
            
            let new_stable_share_asset_value=v_share_asset_value.checked_mul(final_lender_rate).unwrap().checked_div(BASE_MULTIPLIER).unwrap();
            let new_stable_share_liabi_value=v_share_liabi_value.checked_mul(final_borrowing_rate).unwrap().checked_div(BASE_MULTIPLIER).unwrap();
            
            msg!("Log old asset values : asset {}/1000000000000 liability {}/1000000000000",v_share_asset_value,v_share_liabi_value);
            
            msg!("Log new asset values : asset {}/1000000000000 liability {}/1000000000000",new_stable_share_asset_value,new_stable_share_liabi_value);
            
            self.stable_share_asset_value=new_stable_share_asset_value as u64;
            self.volatile_share_liabi_value=new_stable_share_liabi_value as u64;
            
            Ok(())
    }
  
}