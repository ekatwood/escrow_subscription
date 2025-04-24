use anchor_lang::prelude::*;
use crate::state::PlatformConfig;
use crate::error::SubscriptionError;

#[derive(Accounts)]
pub struct UpdatePlatformFeeWallet<'info> {
    #[account(
        mut,
        seeds = [b"platform-config"],
        bump = platform_config.bump,
        has_one = admin
    )]
    pub platform_config: Account<'info, PlatformConfig>,

    pub admin: Signer<'info>,
}

pub fn handler(
    ctx: Context<UpdatePlatformFeeWallet>,
    new_fee_wallet: Pubkey,
) -> Result<()> {
    ctx.accounts.platform_config.fee_wallet = new_fee_wallet;
    Ok(())
}
