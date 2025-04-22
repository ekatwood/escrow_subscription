use anchor_lang::prelude::*;

pub mod instructions;
pub mod state;
pub mod error;

use instructions::*;

declare_id!("Subscr1pt1on11111111111111111111111111111111");

#[program]
pub mod subscription_manager {
    use super::*;

    pub fn initialize_subscription(
        ctx: Context<initialize_subscription::InitializeSubscription>,
        monthly_amount: u64,
        fee_wallet: Pubkey,
    ) -> Result<()> {
        initialize_subscription::handler(ctx, monthly_amount, fee_wallet)
    }

    pub fn process_payment(ctx: Context<process_payment::ProcessPayment>) -> Result<()> {
        process_payment::handler(ctx)
    }

    pub fn cancel_subscription(ctx: Context<cancel_subscription::CancelSubscription>) -> Result<()> {
        cancel_subscription::handler(ctx)
    }

    pub fn update_fee_wallet(
        ctx: Context<update_fee_wallet::UpdateFeeWallet>,
        new_fee_wallet: Pubkey,
    ) -> Result<()> {
        update_fee_wallet::handler(ctx, new_fee_wallet)
    }

    pub fn stake_escrow(ctx: Context<stake_escrow::StakeEscrow>) -> Result<()> {
        stake_escrow::handler(ctx)
    }
}
