use anchor_lang::prelude::*;
use anchor_lang::solana_program;
use anchor_lang::solana_program::clock::UnixTimestamp;
use anchor_spl::associated_token;
use anchor_spl::associated_token::Create;
use crate::errors::ErrorCode;

#[derive(AnchorSerialize, AnchorDeserialize, Debug, Clone)]
pub struct InitTokenParams {
    pub name: String,
    pub symbol: String,
    pub uri: String,
    pub decimals: u8,
}

#[derive(Copy, Clone, AnchorSerialize, AnchorDeserialize)]
pub enum FundingRaiseTier {
    TwentySol,
    FiftySol,
    OneHundredSol,
}

impl FundingRaiseTier {
    pub fn value(&self) -> u64 {
        match self {
            FundingRaiseTier::TwentySol => 20000000000,
            FundingRaiseTier::FiftySol => 50000000000,
            FundingRaiseTier::OneHundredSol => 100000000000,
        }
    }
}

#[derive(Copy, Clone, PartialEq, AnchorSerialize, AnchorDeserialize)]
pub enum LaunchStatus {
    Ongoing,
    Failed,
    Succeed,
}

#[account]
pub struct MemecoinConfig {
    pub creator: Pubkey,
    pub creator_memecoin_index: u32,
    pub created_time: u64,
    pub funding_raise_tier: FundingRaiseTier,
    pub status: LaunchStatus,
}

pub const MEMECOIN_TOTAL_SUPPLY: u64 = 1_000_000_000_000_000;
pub const MEMECOIN_DECIMAL: u64 = 1_000_000;

impl MemecoinConfig {
    pub const LEN: usize = 8 + 32 + 4 + 8 + (1 + 1) + (1 + 1);

    pub fn create_memecoin_config(
        &mut self,
        creator: &Pubkey,
        creator_memecoin_index: u32,
        created_time: u64,
        funding_raise_tier: FundingRaiseTier,
    ) -> Result<()> {
        self.creator = *creator;
        self.creator_memecoin_index = creator_memecoin_index;
        self.created_time = created_time;
        self.funding_raise_tier = funding_raise_tier;
        self.status = LaunchStatus::Ongoing;

        Ok(())
    }

    pub fn token_price(
        &self,
    ) -> Result<u64> {
        let price = self.funding_raise_tier.value()
            .checked_mul(MEMECOIN_DECIMAL).ok_or_else(|| ErrorCode::CalculationError)?
            .checked_div(MEMECOIN_TOTAL_SUPPLY).ok_or_else(|| ErrorCode::CalculationError)?;

        Ok(price)
    }

    pub fn set_memecoin_status(
        &mut self,
        status: LaunchStatus,
    ) -> Result<()> {
        self.status = status;

        Ok(())
    }

}
