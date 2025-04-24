use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Transfer, Mint};
use crate::state::Subscription;
use crate::error::SubscriptionError;

#[derive(Accounts)]
pub struct CancelSubscription<'info> {
    #[account(mut, has_one = user)]
    pub subscription: Account<'info, Subscription>,

    #[account(mut)]
    pub user: Signer<'info>,

    #[account(
        mut,
        seeds = [b"subscription", user.key().as_ref()],
        bump = subscription.bump
    )]
    pub subscription_signer: AccountInfo<'info>,

    #[account(mut)]
    pub escrow_token_account: Account<'info, TokenAccount>,

    #[account(mut)]
    pub destination_token_account: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,
}

pub fn handler(ctx: Context<CancelSubscription>) -> Result<()> {
    let subscription = &mut ctx.accounts.subscription;

    if !subscription.is_active {
        return Err(SubscriptionError::SubscriptionInactive.into());
    }

    let remaining_balance = ctx.accounts.escrow_token_account.amount;

    if remaining_balance > 0 {
        let cpi_accounts = Transfer {
            from: ctx.accounts.escrow_token_account.to_account_info(),
            to: ctx.accounts.destination_token_account.to_account_info(),
            authority: ctx.accounts.subscription_signer.clone(),
        };

        let seeds = &[
            b"subscription",
            ctx.accounts.user.key.as_ref(),
            &[subscription.bump],
        ];
        let signer = &[&seeds[..]];

        token::transfer(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                cpi_accounts,
                signer,
            ),
            remaining_balance,
        )?;
    }

    subscription.is_active = false;

    emit!(SubscriptionCanceled {
        user: ctx.accounts.user.key(),
        refunded_amount: remaining_balance,
        timestamp: Clock::get()?.unix_timestamp,
    });

    Ok(())
}

#[event]
pub struct SubscriptionCanceled {
    pub user: Pubkey,
    pub refunded_amount: u64,
    pub timestamp: i64,
}
