use anchor_lang::prelude::*;
use anchor_spl::token::TokenAccount;

#[account]
pub struct Subscription {
    // The user that owns the subscription
    pub user: Pubkey,

    // The escrow account holding the USDC for the subscription
    #[account(mut)]
    pub escrow_token_account: Pubkey,

    // The amount of USDC for the subscription (per month)
    pub monthly_amount: u64,

    // A flag to indicate if the subscription is active or paused
    pub is_active: bool,

    // The bump seed used for creating a valid PDA for the subscription signer
    pub bump: u8,

    // Platform fee wallet, where the fee for the subscription will be sent
    pub fee_wallet: Pubkey,

    // Optional: Time of last successful payment
    pub last_payment_timestamp: Option<i64>,

    // Optional: Subscription expiration timestamp (if any)
    pub expiration_timestamp: Option<i64>,

    // Optionally, store staked SOL balance if staking is enabled
    pub staked_balance: Option<u64>,
}

impl Subscription {
    // Initializes a new Subscription account with given parameters
    pub fn new(
        user: Pubkey,
        escrow_token_account: Pubkey,
        monthly_amount: u64,
        fee_wallet: Pubkey,
        bump: u8,
    ) -> Self {
        Subscription {
            user,
            escrow_token_account,
            monthly_amount,
            is_active: true, // Subscription is active by default
            bump,
            fee_wallet,
            last_payment_timestamp: None,
            expiration_timestamp: None,
            staked_balance: None,
        }
    }
}
