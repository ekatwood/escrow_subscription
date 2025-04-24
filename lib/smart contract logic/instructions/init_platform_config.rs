use anchor_lang::prelude::*;
use crate::state::PlatformConfig;

#[derive(Accounts)]
#[instruction()]
pub struct InitPlatformConfig<'info> {
    #[account(
        init,
        seeds = [b"platform-config"],
        bump,
        payer = admin,
        space = 8 + PlatformConfig::LEN
    )]
    pub platform_config: Account<'info, PlatformConfig>,

    #[account(mut)]
    pub admin: Signer<'info>,

    pub system_program: Program<'info, System>,
}

pub fn handler(
    ctx: Context<InitPlatformConfig>,
    fee_wallet: Pubkey,
) -> Result<()> {
    let bump = *ctx.bumps.get("platform_config").unwrap();

    ctx.accounts.platform_config.set_inner(PlatformConfig {
        fee_wallet,
        admin: ctx.accounts.admin.key(),
        bump,
    });

    Ok(())
}
