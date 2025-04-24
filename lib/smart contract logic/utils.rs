use anchor_lang::prelude::*;
use anchor_spl::token::{TokenAccount};
use crate::state::Subscription;
use crate::error::SubscriptionError;
use crate::state::platform_config::PlatformConfig;

/// Check if the current signer is the subscription owner
pub fn is_subscription_owner(subscription: &Subscription, signer: &AccountInfo) -> Result<()> {
    if &subscription.user != signer.key {
        return Err(SubscriptionError::Unauthorized.into());
    }
    Ok(())
}

/// Check if the provided account is a valid escrow account
pub fn is_valid_escrow_account(escrow_account: &Account<TokenAccount>) -> Result<()> {
    if escrow_account.amount == 0 {
        return Err(SubscriptionError::InsufficientFunds.into());
    }
    Ok(())
}

/// Validates that the subscription has been initialized
pub fn validate_subscription_initialized(subscription: &Subscription) -> Result<()> {
    if !subscription.is_active {
        return Err(SubscriptionError::SubscriptionInactive.into());
    }
    Ok(())
}

/// Validates that the current transaction has the correct authority
pub fn check_authority(subscription: &Subscription, signer: &AccountInfo) -> Result<()> {
    let seeds = &[b"subscription", subscription.user.as_ref()];
    let (pda, _bump) = Pubkey::find_program_address(seeds, &crate::ID);

    if signer.key != &pda {
        return Err(SubscriptionError::Unauthorized.into());
    }
    Ok(())
}

/// Check if the provided platform fee wallet matches the admin
pub fn is_valid_fee_wallet(platform_config: &PlatformConfig, signer: &AccountInfo) -> Result<()> {
    if platform_config.admin != *signer.key {
        return Err(SubscriptionError::UnauthorizedFeeWalletUpdate.into());
    }
    Ok(())
}

/// Helper to derive the program-derived address (PDA) for a subscription
pub fn get_subscription_pda(subscription: &Subscription) -> Pubkey {
    let seeds = &[b"subscription", subscription.user.as_ref()];
    Pubkey::find_program_address(seeds, &crate::ID).0
}

/// Ensure that the fee wallet has been properly initialized
pub fn validate_fee_wallet_initialized(platform_config: &PlatformConfig) -> Result<()> {
    if platform_config.fee_wallet == Pubkey::default() {
        return Err(SubscriptionError::InvalidSubscriptionState.into());
    }
    Ok(())
}
