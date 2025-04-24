use anchor_lang::prelude::*;

/// Custom error types for the subscription program.
#[error_code]
pub enum SubscriptionError {
    #[msg("The subscription is inactive.")]
    SubscriptionInactive, // Subscription is inactive or paused.

    #[msg("Insufficient funds in the escrow account.")]
    InsufficientFunds, // Not enough USDC in the escrow account to process the payment.

    #[msg("The subscription is already paused.")]
    SubscriptionAlreadyPaused, // User is trying to pause an already paused subscription.

    #[msg("The subscription has already been canceled.")]
    SubscriptionAlreadyCanceled, // User is trying to cancel an already canceled subscription.

    #[msg("Unauthorized access.")]
    Unauthorized, // Attempting to access a resource without the correct authority.

    #[msg("Invalid subscription state.")]
    InvalidSubscriptionState, // The subscription is in an invalid state for the requested action.

    #[msg("Insufficient gas fee funds.")]
    InsufficientGasFeeFunds, // Not enough SOL for transaction fees.

    #[msg("The platform fee wallet cannot be updated.")]
    UnauthorizedFeeWalletUpdate, // Only admin can update the platform fee wallet.

    #[msg("Invalid amount specified.")]
    InvalidAmount, // An invalid amount (e.g., negative or zero) was passed to a transaction.

    #[msg("Escrow account does not exist.")]
    EscrowAccountNotFound, // The escrow account was not found or doesn't exist.

    #[msg("Subscription already exists.")]
    SubscriptionAlreadyExists, // A subscription for this user already exists.

    #[msg("Unable to stake the escrow funds.")]
    StakeFailed, // The process of staking failed.

    #[msg("Unable to unstake the escrow funds.")]
    UnstakeFailed, // The process of unstaking failed.
}
