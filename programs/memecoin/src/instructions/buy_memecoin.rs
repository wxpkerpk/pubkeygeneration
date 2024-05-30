use anchor_lang::prelude::*;
use crate::state::*;
use anchor_lang::{
    solana_program::{
        clock::UnixTimestamp,
        sysvar::clock::Clock,
        system_instruction::transfer as lamports_transfer,
        program::invoke,
    }
};
use anchor_spl::{
    associated_token::AssociatedToken,
    metadata::{create_metadata_accounts_v3, CreateMetadataAccountsV3, Metadata},
    token::{transfer as memecoin_transfer, Burn, Mint, Token, TokenAccount, Transfer},
    //token_2022::{self, transfer_checked as memecoin_transfer, TransferChecked, Token2022},
};
use anchor_spl::token_interface::TokenInterface;
use crate::errors::ErrorCode;

#[derive(Accounts)]
pub struct BuyMemecoin<'info> {
    #[account(
        mut,
        seeds = [memecoin_config.creator.key().as_ref(), &memecoin_config.creator_memecoin_index.to_le_bytes()],
        bump
    )]
    pub memecoin_config: Account<'info, MemecoinConfig>,

    #[account(
        mut,
        seeds = [b"mint", memecoin_config.key().as_ref()],
        bump,
    )]
    pub mint: Account<'info, Mint>,

    #[account(mut)]
    pub buyer: Signer<'info>,

    #[account(
        init_if_needed,
        payer = buyer,
        token::mint = mint,
        token::authority = buyer,
        seeds=[b"MEME_COIN", mint.key().as_ref(), buyer.key().as_ref()],
        bump
    )]
    pub buyer_token: Account<'info, TokenAccount>,

    #[account(
        mut,
        token::mint = mint,
        token::authority = memecoin_config,
        seeds=[b"MEME_COIN", mint.key().as_ref(), memecoin_config.key().as_ref()],
        bump
    )]
    pub memecoin_config_token: Account<'info, TokenAccount>,

    pub clock: Sysvar<'info, Clock>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
    /// Spl token program or token program 2022
    pub token_2022_program: Interface<'info, TokenInterface>,
}

#[event]
pub struct MemecoinBought {
    pub buyer: Pubkey,
    pub buy_amount: u64,
    pub mint: Pubkey,
    pub token_price: u64,
    pub remain_amount: u64, // Remaining amount to sell
}

pub fn handler(
    ctx: Context<BuyMemecoin>,
    buy_amount: u64,
) -> Result<()> {
    require!(ctx.accounts.memecoin_config.status == LaunchStatus::Ongoing, ErrorCode::StatusNotOngoing);

    let memecoin_config_token_balance = ctx.accounts.memecoin_config_token.amount;

    let sold_amount = MEMECOIN_TOTAL_SUPPLY
        .checked_sub(memecoin_config_token_balance)
        .ok_or_else(|| ErrorCode::CalculationError)?;
    require!(sold_amount + buy_amount <= MEMECOIN_TOTAL_SUPPLY / 2, ErrorCode::UnsoldTokenInsufficient);

    let current_timestamp = ctx.accounts.clock.unix_timestamp as u64;
    let memecoin_created_time = ctx.accounts.memecoin_config.created_time;
    let memecoin_config = &mut ctx.accounts.memecoin_config;
    if current_timestamp >= memecoin_created_time + 3600 {
        if sold_amount == MEMECOIN_TOTAL_SUPPLY / 2 {
            memecoin_config.set_memecoin_status(
                LaunchStatus::Succeed
            )?;
        } else {
            memecoin_config.set_memecoin_status(
                LaunchStatus::Failed
            )?;
        }

        return err!(ErrorCode::SaleClosed);
    } else {
        if sold_amount + buy_amount == MEMECOIN_TOTAL_SUPPLY / 2 {
            memecoin_config.set_memecoin_status(
                LaunchStatus::Succeed
            )?;
        }
    }

    // User pay for the memecoin by lamports
    let token_price = ctx.accounts.memecoin_config.token_price()?;
    let cost = buy_amount.checked_mul(token_price).ok_or_else(|| ErrorCode::CalculationError)?;
    let transfer_instruction = lamports_transfer(
        &ctx.accounts.buyer.key(),
        &ctx.accounts.memecoin_config.key(),
        cost
    );
    invoke(
        &transfer_instruction,
        &[
            ctx.accounts.buyer.to_account_info(),
            ctx.accounts.memecoin_config.to_account_info(),
            ctx.accounts.system_program.to_account_info(),
        ],
    )?;

    // Send user the memecoin
    let seeds = &[
        ctx.accounts.memecoin_config.creator.as_ref(),
        &ctx.accounts.memecoin_config.creator_memecoin_index.to_le_bytes(),
        &[ctx.bumps.memecoin_config]
    ];
    let signer = [&seeds[..]];

    memecoin_transfer(
        CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: ctx.accounts.memecoin_config_token.to_account_info(),
                to: ctx.accounts.buyer_token.to_account_info(),
                authority: ctx.accounts.memecoin_config.to_account_info(),
            },
            &signer,
        ),
        buy_amount,
    )?;

    let remain_amount = (MEMECOIN_TOTAL_SUPPLY / 2)
        .checked_sub(sold_amount).unwrap()
        .checked_sub(buy_amount).unwrap();
    emit!(MemecoinBought {
            buyer: ctx.accounts.buyer.key(),
            buy_amount,
            mint: ctx.accounts.mint.key(),
            token_price,
            remain_amount,
        }
    );

    Ok(())
}