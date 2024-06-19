use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{self, Token, Burn, Transfer, Mint, TokenAccount, transfer as memecoin_transfer},
    token_2022::{self, TransferChecked, Token2022},
};
use anchor_lang::{
    solana_program::system_instruction::transfer as lamports_transfer,
    solana_program::pubkey::Pubkey,
};
use crate::state::{MemecoinConfig, LaunchStatus, GlobalConfig, MEMECOIN_TOTAL_SUPPLY};
use crate::errors::ErrorCode;
use std::str::FromStr;
use crate::constants::WSOL_MINT_ADDRESS;
use crate::constants::CREATE_RAYDIUM_POOL_FEE;

#[derive(Accounts)]
pub struct CreateRaydiumPoolByAdmin<'info> {
    #[account(
        mut,
        seeds = [memecoin_config.creator.key().as_ref(), &memecoin_config.creator_memecoin_index.to_le_bytes()],
        bump
    )]
    pub memecoin_config: Account<'info, MemecoinConfig>,

    #[account(
        mut,
        has_one = admin,
        seeds = [b"CONFIG"],
        bump
    )]
    pub global_config: Account<'info, GlobalConfig>,

    #[account(mut)]
    pub admin: Signer<'info>,

    #[account(
        mut,
        seeds = [b"mint", memecoin_config.key().as_ref()],
        bump,
    )]
    pub memecoin_config_mint: Account<'info, Mint>,

    /// CHECK：checked in the handler
    pub wsol_mint: Account<'info, Mint>,

    #[account(
        mut,
        token::mint = memecoin_config_mint,
        token::authority = memecoin_config,
        seeds=[b"MEME_COIN", memecoin_config_mint.key().as_ref(), memecoin_config.key().as_ref()],
        bump
    )]
    pub memecoin_config_token: Account<'info, TokenAccount>,

    #[account(
        mut,
        token::mint = wsol_mint,
        token::authority = memecoin_config,
        seeds=[b"WSOL", memecoin_config.key().as_ref()],
        bump
    )]
    pub memecoin_config_wsol_token: Account<'info, TokenAccount>,

    #[account(mut)]
    pub admin_memecoin_token: UncheckedAccount<'info>,

    #[account(mut)]
    pub admin_wsol_token: UncheckedAccount<'info>,

    /// CHECK: checked by address constraint
    #[account(
        mut,
        address = global_config.launch_success_fee_receiver.key(),
    )]
    pub launch_success_fee_receiver: UncheckedAccount<'info>,

    /// CHECK: checked by address constraint
    #[account(
        mut,
    )]
    pub launch_success_memecoin_fee_receiver: UncheckedAccount<'info>,

    /// CHECK: checked by address constraint
    #[account(
        mut,
    )]
    pub launch_success_wsol_fee_receiver: Account<'info, TokenAccount>,

    /// Program to create mint account and mint tokens
    pub token_program: Program<'info, Token>,
    /// Program to create an ATA for receiving position NFT
    pub associated_token_program: Program<'info, AssociatedToken>,
    /// To create a new program account
    pub system_program: Program<'info, System>,
    /// Sysvar for program account
    pub rent: Sysvar<'info, Rent>,
}

pub fn handler(ctx: Context<CreateRaydiumPoolByAdmin>) -> Result<()> {
    require!(ctx.accounts.memecoin_config.status == LaunchStatus::Succeed, ErrorCode::OnlyCreatePoolWhenLaunchSuccess);
    let wsol_mint_pubkey = Pubkey::from_str(WSOL_MINT_ADDRESS).unwrap();
    require_keys_eq!(ctx.accounts.wsol_mint.key(), wsol_mint_pubkey, ErrorCode::WrongWSOLMint);

    let total_funding_raise_amount = ctx.accounts.memecoin_config.funding_raise_tier.value();
    let launch_success_fee_bps = ctx.accounts.global_config.launch_success_fee_bps as u64;
    let mut launch_success_fee_sol_amount = total_funding_raise_amount
        .checked_mul(launch_success_fee_bps).ok_or_else(|| ErrorCode::CalculationError)?
        .checked_div(10000u64).ok_or_else(|| ErrorCode::CalculationError)?;
    let transfer_to_admin_wsol_amount = total_funding_raise_amount
        .checked_sub(launch_success_fee_sol_amount)
        .ok_or_else(|| ErrorCode::CalculationError)?
        .checked_sub(CREATE_RAYDIUM_POOL_FEE)
        .ok_or_else(|| ErrorCode::CalculationError)?;

    let launch_success_fee_memecoin_amount = ctx.accounts.memecoin_config_token.amount
        .checked_mul(launch_success_fee_bps).ok_or_else(|| ErrorCode::CalculationError)?
        .checked_div(10000u64).ok_or_else(|| ErrorCode::CalculationError)?;
    let transfer_to_admin_memecoin_amount = ctx.accounts.memecoin_config_token.amount
        .checked_sub(launch_success_fee_memecoin_amount)
        .ok_or_else(|| ErrorCode::CalculationError)?;


    let seeds = &[
        ctx.accounts.memecoin_config.creator.as_ref(),
        &ctx.accounts.memecoin_config.creator_memecoin_index.to_le_bytes(),
        &[ctx.bumps.memecoin_config]
    ];
    let signer = [&seeds[..]];

    // Transfer wsol to admin
    token::transfer(
        CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: ctx.accounts.memecoin_config_wsol_token.to_account_info(),
                to: ctx.accounts.admin_wsol_token.to_account_info(),
                authority: ctx.accounts.memecoin_config.clone().to_account_info(),
            },
            &signer,
        ),
        transfer_to_admin_wsol_amount,
    )?;
    msg!("transfer to admin wsol amount is : {}", transfer_to_admin_wsol_amount);

    // Transfer memecoin to admin
    token::transfer(
        CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: ctx.accounts.memecoin_config_token.to_account_info(),
                to: ctx.accounts.admin_memecoin_token.to_account_info(),
                authority: ctx.accounts.memecoin_config.clone().to_account_info(),
            },
            &signer,
        ),
        transfer_to_admin_memecoin_amount,
    )?;
    msg!("transfer to admin memecoin amount is : {}", transfer_to_admin_memecoin_amount);

    // Transfer memecoin fee
    token::transfer(
        CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: ctx.accounts.memecoin_config_token.to_account_info(),
                to: ctx.accounts.launch_success_memecoin_fee_receiver.to_account_info(),
                authority: ctx.accounts.memecoin_config.clone().to_account_info(),
            },
            &signer,
        ),
        launch_success_fee_memecoin_amount,
    )?;
    msg!("launch success fee memecoin amount is : {}", launch_success_fee_memecoin_amount);

    // Transfer sol fee
    let rent = Rent::get()?;
    let account_size = ctx.accounts.memecoin_config.to_account_info().data_len();
    let rent_exempt_minimum = rent.minimum_balance(account_size);

    // Calculate the amount of lamports to transfer
    let memecoin_config_account_balance = **ctx.accounts.memecoin_config.to_account_info().lamports.borrow();
    let lamports_to_transfer = memecoin_config_account_balance.saturating_sub(rent_exempt_minimum);
    if launch_success_fee_sol_amount > lamports_to_transfer {
        launch_success_fee_sol_amount = lamports_to_transfer;
    }

    ctx.accounts.memecoin_config.sub_lamports(launch_success_fee_sol_amount)?;
    ctx.accounts.launch_success_fee_receiver.add_lamports(launch_success_fee_sol_amount)?;
    msg!("launch success fee sol amount is : {}", launch_success_fee_sol_amount);

    // Transfer create raydium pool fee
    ctx.accounts.memecoin_config.sub_lamports(CREATE_RAYDIUM_POOL_FEE)?;
    ctx.accounts.admin.add_lamports(CREATE_RAYDIUM_POOL_FEE)?;
    msg!("create raydium pool fee is : {}", CREATE_RAYDIUM_POOL_FEE);

    Ok(())
}