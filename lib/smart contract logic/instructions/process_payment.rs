use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Transfer, Mint};
use crate::state::{Subscription, PlatformConfig};
use crate::error::SubscriptionError;

const FEE_AMOUNT_USDC: u64 = 10_000; // 0.10 USDC with 6 decimals

#[derive(Accounts)]
pub struct ProcessPayment<'info> {
    #[account(mut)]
    pub subscription: Account<'info, Subscription>,

    #[account(
        mut,
        seeds = [b"subscription", subscription.user.as_ref()],
        bump = subscription.bump,
        has_one = user
    )]
    pub subscription_signer: AccountInfo<'info>,

    /// CHECK: Just storing and verifying keys
    pub user: AccountInfo<'info>,

    #[account(mut)]
    pub escrow_token_account: Account<'info, TokenAccount>,

    #[account(mut)]
    pub recipient_token_account: Account<'info, TokenAccount>,

    #[account(mut)]
    pub fee_wallet_token_account: Account<'info, TokenAccount>,

    #[account(
        seeds = [b"platform-config"],
        bump
    )]
    pub platform_config: Account<'info, PlatformConfig>,

    pub usdc_mint: Account<'info, Mint>,

    pub token_program: Program<'info, Token>,
}

pub fn handler(ctx: Context<ProcessPayment>) -> Result<()> {
    let subscription = &ctx.accounts.subscription;

    if !subscription.is_active {
        return Err(SubscriptionError::SubscriptionInactive.into());
    }

    let total_required = subscription.monthly_amount + FEE_AMOUNT_USDC;
    let escrow_balance = ctx.accounts.escrow_token_account.amount;

    require!(
        escrow_balance >= total_required,
        SubscriptionError::InsufficientFunds
    );

    let seeds = &[
        b"subscription",
        subscription.user.as_ref(),
        &[subscription.bump],
    ];
    let signer = &[&seeds[..]];

    // Step 1: Transfer main payment to recipient
    token::transfer(
        CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: ctx.accounts.escrow_token_account.to_account_info(),
                to: ctx.accounts.recipient_token_account.to_account_info(),
                authority: ctx.accounts.subscription_signer.clone(),
            },
            signer,
        ),
        subscription.monthly_amount,
    )?;

    // Step 2: Transfer $0.10 USDC fee to platform
    token::transfer(
        CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: ctx.accounts.escrow_token_account.to_account_info(),
                to: ctx.accounts.fee_wallet_token_account.to_account_info(),
                authority: ctx.accounts.subscription_signer.clone(),
            },
            signer,
        ),
        FEE_AMOUNT_USDC,
    )?;

    emit!(PaymentProcessed {
        user: subscription.user,
        amount: subscription.monthly_amount,
        fee_wallet: ctx.accounts.platform_config.fee_wallet,
        timestamp: Clock::get()?.unix_timestamp,
    });

    Ok(())
}

#[event]
pub struct PaymentProcessed {
    pub user: Pubkey,
    pub amount: u64,
    pub fee_wallet: Pubkey,
    pub timestamp: i64,
}
