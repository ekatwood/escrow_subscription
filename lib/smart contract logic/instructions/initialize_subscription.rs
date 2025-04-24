use anchor_lang::prelude::*;
use crate::state::Subscription;

#[derive(Accounts)]
#[instruction(monthly_amount: u64)]
pub struct InitializeSubscription<'info> {
    #[account(
        init,
        payer = user,
        space = Subscription::LEN,
        seeds = [b"subscription", user.key().as_ref()],
        bump
    )]
    pub subscription: Account<'info, Subscription>,

    #[account(mut)]
    pub user: Signer<'info>,

    /// CHECK: fee_wallet is saved as a pubkey, not used for execution logic directly
    pub fee_wallet: UncheckedAccount<'info>,

    pub system_program: Program<'info, System>,
}

pub fn handler(
    ctx: Context<InitializeSubscription>,
    monthly_amount: u64,
    fee_wallet: Pubkey,
) -> Result<()> {
    let subscription = &mut ctx.accounts.subscription;

    subscription.user = ctx.accounts.user.key();
    subscription.monthly_amount = monthly_amount;
    subscription.fee_wallet = fee_wallet;
    subscription.bump = *ctx.bumps.get("subscription").unwrap();
    subscription.is_active = true;

    Ok(())
}
