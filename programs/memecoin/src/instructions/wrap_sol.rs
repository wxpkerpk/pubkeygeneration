use anchor_lang::prelude::*;
use crate::state::*;
use anchor_lang::{
    solana_program::{
        clock::UnixTimestamp,
        sysvar::clock::Clock,
        system_instruction::transfer as lamports_transfer,
    }
};
use anchor_spl::{
    associated_token::AssociatedToken,
    token_2022::{self, Token2022, Transfer, transfer_checked},
    token::{self, Token, TokenAccount, Mint}
};
use crate::errors::ErrorCode;
use std::str::FromStr;
use crate::constants::WSOL_MINT_ADDRESS;

#[derive(Accounts)]
pub struct WrapSol<'info> {
    #[account(
        mut,
        seeds = [memecoin_config.creator.key().as_ref(), &memecoin_config.creator_memecoin_index.to_le_bytes()],
        bump
    )]
    pub memecoin_config: Account<'info, MemecoinConfig>,

    #[account(
        seeds = [b"CONFIG"],
        bump
    )]
    pub global_config: Account<'info, GlobalConfig>,

    #[account(mut)]
    pub payer: Signer<'info>,

    pub wrapped_sol_mint: Account<'info, Mint>,

    #[account(
        init_if_needed,
        payer = payer,
        token::mint = wrapped_sol_mint,
        token::authority = memecoin_config,
        seeds=[b"WSOL", memecoin_config.key().as_ref()],
        bump
    )]
    pub memecoin_config_wrapped_sol_account: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
    pub token_program_2022: Program<'info, Token2022>,
}

pub fn handler(
    ctx: Context<WrapSol>,
) -> Result<()> {
    require!(ctx.accounts.memecoin_config.status == LaunchStatus::Succeed, ErrorCode::OnlyCreatePoolWhenLaunchSuccess);

    let wsol_mint_pubkey = Pubkey::from_str(WSOL_MINT_ADDRESS).unwrap();
    require_keys_eq!(ctx.accounts.wrapped_sol_mint.key(), wsol_mint_pubkey, ErrorCode::WrongWSOLMint);

    let seeds = &[
        ctx.accounts.memecoin_config.creator.as_ref(),
        &ctx.accounts.memecoin_config.creator_memecoin_index.to_le_bytes(),
        &[ctx.bumps.memecoin_config]
    ];
    let signer = [&seeds[..]];

    // Transfer SOL to the new WSOL account
    let total_funding_raise_amount = ctx.accounts.memecoin_config.funding_raise_tier.value();
    let launch_success_fee_bps = ctx.accounts.global_config.launch_success_fee_bps as u64;
    let wrap_amount = total_funding_raise_amount
        .checked_mul(
            10000u64.checked_sub(launch_success_fee_bps).ok_or_else(|| ErrorCode::CalculationError)?
        ).ok_or_else(|| ErrorCode::CalculationError)?
        .checked_div(10000u64).ok_or_else(|| ErrorCode::CalculationError)?;

    lamports_transfer(
        &ctx.accounts.memecoin_config.key(),
        &ctx.accounts.memecoin_config_wrapped_sol_account.key(),
        wrap_amount
    );

    // Initialize the WSOL account
    let cpi_accounts = token::InitializeAccount3 {
        account: ctx.accounts.memecoin_config_wrapped_sol_account.to_account_info(),
        mint: ctx.accounts.wrapped_sol_mint.to_account_info(),
        authority: ctx.accounts.memecoin_config.to_account_info(),
    };

    let cpi_ctx = CpiContext::new_with_signer(
        ctx.accounts.token_program.to_account_info(),
        cpi_accounts,
        &signer
    );
    token::initialize_account3(cpi_ctx)?;

    Ok(())
}