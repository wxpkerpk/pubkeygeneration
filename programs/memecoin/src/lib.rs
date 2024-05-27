use anchor_lang::prelude::*;
use instructions::*;
use state::{InitTokenParams, FundingRaiseTier};

declare_id!("AGAqYVPTrydtAdzLQBt8c1Rp61dHNipmsiCPHyYgMzV");

pub mod errors;
pub mod constants;
pub mod instructions;
pub mod state;

#[program]
pub mod memecoin {
    use super::*;

    /* ====================================== Admin Instructions ============================================ */

    pub fn initialize_global_configs(
        ctx: Context<InitializeGlobalConfig>,
        create_memecoin_fee_receiver: Pubkey,
        launch_success_fee_receiver: Pubkey,
        create_memecoin_fee: u64,
        launch_success_fee_bps: u16,
    ) -> Result<()> {
        return initialize_global_config::handler(
            ctx,
            create_memecoin_fee_receiver,
            launch_success_fee_receiver,
            create_memecoin_fee,
            launch_success_fee_bps
        );
    }

    pub fn set_create_memecoin_fee_receiver(
        ctx: Context<SetCreateMemecoinFeeReceiver>,
        create_memecoin_fee_receiver: Pubkey
    ) -> Result<()> {
        return set_create_memecoin_fee_receiver::handler(ctx, &create_memecoin_fee_receiver);
    }

    pub fn set_launch_success_fee_receiver(
        ctx: Context<SetLaunchSuccessFeeReceiver>,
        launch_success_fee_receiver: Pubkey
    ) -> Result<()> {
        return set_launch_success_fee_receiver::handler(ctx, &launch_success_fee_receiver);
    }

    pub fn set_create_memecoin_fee(
        ctx: Context<SetCreateMemecoinFee>,
        create_memecoin_fee: u64
    ) -> Result<()> {
        return set_create_memecoin_fee::handler(ctx, create_memecoin_fee);
    }

    pub fn set_launch_success_fee_bps(
        ctx: Context<SetLaunchSuccessFeeBps>,
        launch_success_fee_bps: u16
    ) -> Result<()> {
        return set_launch_success_fee_bps::handler(ctx, launch_success_fee_bps);
    }

    /* ====================================== User Instructions ============================================ */

    pub fn create_memecoin_config(
        ctx: Context<CreateMemecoinConfig>,
        memecoin_name: String,
        memecoin_symbol: String,
        memecoin_uri: String,
        memecoin_decimals: u8,
        funding_raise_tier: u8
    ) -> Result<()> {
        return create_memecoin_config::handler(
            ctx,
            &memecoin_name,
            &memecoin_symbol,
            &memecoin_uri,
            memecoin_decimals,
            funding_raise_tier,
        );
    }

    pub fn buy_memecoin(
        ctx: Context<BuyMemecoin>,
        buy_amount: u64
    ) -> Result<()> {
        return buy_memecoin::handler(ctx, buy_amount);
    }

    pub fn claim_lamports(
        ctx: Context<ClaimLamports>,
        claim_amount: u64
    ) -> Result<()> {
        return claim_lamports::handler(ctx, claim_amount);
    }

    pub fn wrap_sol(
        ctx: Context<WrapSol>,
    ) -> Result<()> {
        return wrap_sol::handler(
            ctx,
        );
    }

    pub fn create_raydium_pool(
        ctx: Context<CreateRaydiumPool>,
    ) -> Result<()> {
        return create_raydium_pool::handler(
            ctx,
        );
    }

}