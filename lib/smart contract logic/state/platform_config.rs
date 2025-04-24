use anchor_lang::prelude::*;

#[account]
pub struct PlatformConfig {
    pub fee_wallet: Pubkey,
    pub admin: Pubkey,
    pub bump: u8,
}

impl PlatformConfig {
    pub const LEN: usize = 32 + 32 + 1; // fee_wallet + admin + bump
}
