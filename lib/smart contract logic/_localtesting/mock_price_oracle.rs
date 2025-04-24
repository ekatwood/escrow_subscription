use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Transfer};
use solana_program::pubkey::Pubkey;

#[derive(Accounts)]
pub struct MockPriceOracle<'info> {
    #[account(mut)]
    pub price_oracle_account: Account<'info, PriceOracle>,
    pub token_program: Program<'info, Token>,
}

#[account]
pub struct PriceOracle {
    pub price: u64, // Price of SOL in USDC with 6 decimals
}

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct SetPriceArgs {
    pub new_price: u64, // New price in 6 decimal format
}

pub fn handler(ctx: Context<MockPriceOracle>, args: SetPriceArgs) -> Result<()> {
    let price_oracle_account = &mut ctx.accounts.price_oracle_account;

    price_oracle_account.price = args.new_price;
    msg!("Updated the price oracle with new price: {}", args.new_price);

    Ok(())
}

#[program]
pub mod mock_price_oracle {
    use super::*;

    pub fn set_price(ctx: Context<MockPriceOracle>, new_price: u64) -> Result<()> {
        let args = SetPriceArgs { new_price };
        handler(ctx, args)
    }

    pub fn get_price(ctx: Context<MockPriceOracle>) -> Result<u64> {
        let price_oracle_account = &ctx.accounts.price_oracle_account;
        Ok(price_oracle_account.price)
    }
}

