use anchor_lang::prelude::*;
use anchor_lang::solana_program::stake;
use anchor_lang::solana_program::stake::instruction as stake_instruction;
use crate::state::Subscription;
use crate::error::SubscriptionError;

#[derive(Accounts)]
pub struct Unstake<'info> {
    #[account(mut)]
    pub subscription: Account<'info, Subscription>,

    #[account(
        mut,
        seeds = [b"subscription", subscription.user.as_ref()],
        bump = subscription.bump
    )]
    /// CHECK: This is the PDA signer of the subscription
    pub subscription_signer: AccountInfo<'info>,

    /// CHECK: The user's stake account to unstake from
    #[account(mut)]
    pub stake_account: AccountInfo<'info>,

    /// CHECK: The user's token account for receiving SOL after unstaking
    #[account(mut)]
    pub recipient: AccountInfo<'info>,

    pub system_program: Program<'info, System>,
    pub clock: Sysvar<'info, Clock>,
}

pub fn handler(ctx: Context<Unstake>) -> Result<()> {
    let subscription = &ctx.accounts.subscription;

    // Ensure subscription is active
    if !subscription.is_active {
        return Err(SubscriptionError::SubscriptionInactive.into());
    }

    // Check that the stake account belongs to the subscription signer
    if &ctx.accounts.stake_account.owner != ctx.accounts.subscription_signer.key {
        return Err(SubscriptionError::InvalidSigner.into());
    }

    // Get the balance of staked SOL
    let stake_balance = ctx.accounts.stake_account.to_account_info().lamports();
    if stake_balance == 0 {
        return Err(SubscriptionError::InsufficientFunds.into());
    }

    // Unstake SOL (withdraw the funds)
    let unstake_ix = stake_instruction::deactivate(
        &ctx.accounts.stake_account.key(),
        &ctx.accounts.subscription_signer.key(),
    );

    anchor_lang::solana_program::program::invoke_signed(
        &unstake_ix,
        &[
            ctx.accounts.stake_account.to_account_info(),
            ctx.accounts.subscription_signer.to_account_info(),
            ctx.accounts.system_program.to_account_info(),
        ],
        &[&[
            b"subscription",
            ctx.accounts.subscription.user.as_ref(),
            &[ctx.accounts.subscription.bump],
        ][..]],
    )?;

    // Transfer unstaked SOL back to the user's recipient account
    let transfer_lamports = ctx.accounts.stake_account.to_account_info().lamports();
    **ctx.accounts.recipient.lamports.borrow_mut() += transfer_lamports;
    **ctx.accounts.stake_account.lamports.borrow_mut() -= transfer_lamports;

    Ok(())
}
