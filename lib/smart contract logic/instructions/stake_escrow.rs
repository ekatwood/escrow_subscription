use anchor_lang::prelude::*;
use anchor_lang::solana_program::stake;
use anchor_lang::solana_program::stake::instruction as stake_instruction;
use anchor_lang::solana_program::stake::state::{Authorized, Lockup};
use anchor_lang::solana_program::system_instruction;
use crate::state::Subscription;
use crate::error::SubscriptionError;

#[derive(Accounts)]
pub struct StakeEscrow<'info> {
    #[account(mut)]
    pub subscription: Account<'info, Subscription>,

    #[account(
        mut,
        seeds = [b"subscription", subscription.user.as_ref()],
        bump = subscription.bump
    )]
    /// CHECK: This is the PDA signer of the subscription
    pub subscription_signer: AccountInfo<'info>,

    /// CHECK: New stake account owned by the subscription
    #[account(mut)]
    pub stake_account: UncheckedAccount<'info>,

    /// CHECK: BlazeStake validator's vote account
    pub validator_vote: UncheckedAccount<'info>,

    #[account(mut)]
    pub system_program: Program<'info, System>,

    pub rent: Sysvar<'info, Rent>,
    pub clock: Sysvar<'info, Clock>,
}

pub fn handler(
    ctx: Context<StakeEscrow>,
    stake_lamports: u64,
) -> Result<()> {
    // Derive signer seeds
    let seeds = &[
        b"subscription",
        ctx.accounts.subscription.user.as_ref(),
        &[ctx.accounts.subscription.bump],
    ];
    let signer_seeds = &[&seeds[..]];

    // Create the stake account
    let create_stake_ix = system_instruction::create_account(
        &ctx.accounts.subscription_signer.key(),
        &ctx.accounts.stake_account.key(),
        stake_lamports,
        stake::state::StakeState::size_of() as u64,
        &stake::program::ID,
    );

    anchor_lang::solana_program::program::invoke_signed(
        &create_stake_ix,
        &[
            ctx.accounts.subscription_signer.clone(),
            ctx.accounts.stake_account.to_account_info(),
            ctx.accounts.system_program.to_account_info(),
        ],
        signer_seeds,
    )?;

    // Initialize stake account
    let authorized = Authorized {
        staker: ctx.accounts.subscription_signer.key(),
        withdrawer: ctx.accounts.subscription_signer.key(),
    };
    let lockup = Lockup::default();

    let init_stake_ix = stake_instruction::initialize(
        &ctx.accounts.stake_account.key(),
        &authorized,
        &lockup,
    );

    anchor_lang::solana_program::program::invoke_signed(
        &init_stake_ix,
        &[
            ctx.accounts.stake_account.to_account_info(),
            ctx.accounts.rent.to_account_info(),
        ],
        signer_seeds,
    )?;

    // Delegate to BlazeStake validator
    let delegate_ix = stake_instruction::delegate_stake(
        &ctx.accounts.stake_account.key(),
        &ctx.accounts.subscription_signer.key(),
        &ctx.accounts.validator_vote.key(),
    );

    anchor_lang::solana_program::program::invoke_signed(
        &delegate_ix,
        &[
            ctx.accounts.stake_account.to_account_info(),
            ctx.accounts.validator_vote.to_account_info(),
            ctx.accounts.clock.to_account_info(),
        ],
        signer_seeds,
    )?;

    Ok(())
}
