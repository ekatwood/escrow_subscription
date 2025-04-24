use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Transfer};
use solana_program::pubkey::Pubkey;
use anchor_spl::associated_token::AssociatedToken;
use crate::program::SubscriptionProgram;
use crate::state::{Subscription, platform_config::PlatformConfig};
use crate::error::SubscriptionError;
use crate::utils::{check_authority, is_subscription_owner, validate_subscription_initialized};
use crate::instruction::{init_subscription, process_payment, cancel_subscription, update_fee_wallet};
use anchor_lang::solana_program::system_program;
use anchor_spl::token::Mint;
use solana_program_test::{ProgramTest, processor};
use solana_sdk::{signature::{Keypair,Signer}, transaction::Transaction, commitment_config::CommitmentConfig, pubkey::Pubkey as SolanaPubkey};

#[derive(Debug)]
pub struct TestContext {
    pub program_test: ProgramTest,
    pub user: Keypair,
    pub platform_admin: Keypair,
    pub subscription_pda: SolanaPubkey,
    pub fee_wallet: SolanaPubkey,
    pub mint: Keypair,
    pub token_program: Pubkey,
}

impl TestContext {
    fn new() -> Self {
        let program_test = ProgramTest::new(
            "subscription_program",
            crate::ID,
            processor!(SubscriptionProgram::process_instruction),
        );

        let user = Keypair::new();
        let platform_admin = Keypair::new();
        let fee_wallet = platform_admin.pubkey();
        let mint = Keypair::new();
        let token_program = anchor_spl::token::ID;

        let subscription_pda = Pubkey::find_program_address(&[b"subscription", user.pubkey().as_ref()], &crate::ID).0;

        TestContext {
            program_test,
            user,
            platform_admin,
            subscription_pda,
            fee_wallet,
            mint,
            token_program,
        }
    }

    fn create_accounts(&self) {
        let mut context = self.program_test.start_with_context().unwrap();

        let user_token_account = self.create_associated_token_account(&mut context, &self.user, &self.mint.pubkey());
        let fee_wallet_account = self.create_associated_token_account(&mut context, &self.platform_admin, &self.mint.pubkey());
        let escrow_account = self.create_associated_token_account(&mut context, &self.user, &self.mint.pubkey());

        let platform_config = PlatformConfig {
            admin: self.platform_admin.pubkey(),
            fee_wallet: self.fee_wallet,
        };

        context.banks_client
            .process_transaction(Transaction::new_signed_with_payer(
                &[platform_config],
                Some(&context.payer.pubkey()),
                &[&self.platform_admin],
                context.last_blockhash,
            ))
            .unwrap();
    }

    fn create_associated_token_account(&self, context: &mut TestContext, owner: &Keypair, mint: &SolanaPubkey) -> SolanaPubkey {
        let associated_token = AssociatedToken::get_associated_token_address(owner.pubkey(), mint);
        // Create associated token accounts for the owner
        let create_associated_token_ix = AssociatedToken::create_associated_token_account(
            context.payer.pubkey(),
            owner.pubkey(),
            mint,
        );

        context.banks_client
            .process_transaction(Transaction::new_signed_with_payer(
                &[create_associated_token_ix],
                Some(&context.payer.pubkey()),
                &[context.payer, owner],
                context.last_blockhash,
            ))
            .unwrap();

        associated_token
    }
}

#[tokio::test]
async fn test_initialize_subscription() {
    let test_ctx = TestContext::new();
    test_ctx.create_accounts();

    let mut context = test_ctx.program_test.start_with_context().unwrap();

    let subscription = Subscription {
        user: test_ctx.user.pubkey(),
        monthly_amount: 10 * 1_000_000,  // 10 USDC, assuming 6 decimals
        is_active: true,
        bump: 0,
    };

    let subscription_pda = test_ctx.subscription_pda;
    let fee_wallet = test_ctx.fee_wallet;

    let transaction = Transaction::new_signed_with_payer(
        &[
            init_subscription(
                &test_ctx.program_test,
                test_ctx.user.pubkey(),
                subscription.monthly_amount,
                fee_wallet,
            ),
        ],
        Some(&context.payer.pubkey()),
        &[&context.payer, &test_ctx.user],
        context.last_blockhash,
    );

    context.banks_client.process_transaction(transaction).unwrap();
    let subscription_data = context.banks_client
        .get_account_data(&subscription_pda)
        .unwrap();

    assert_eq!(subscription_data.user, test_ctx.user.pubkey());
    assert_eq!(subscription_data.monthly_amount, subscription.monthly_amount);
}

#[tokio::test]
async fn test_process_payment() {
    let test_ctx = TestContext::new();
    test_ctx.create_accounts();

    let mut context = test_ctx.program_test.start_with_context().unwrap();

    let payment_amount = 10 * 1_000_000;  // 10 USDC, assuming 6 decimals
    let user_token_account = test_ctx.create_associated_token_account(&mut context, &test_ctx.user, &test_ctx.mint.pubkey());
    let escrow_token_account = test_ctx.create_associated_token_account(&mut context, &test_ctx.user, &test_ctx.mint.pubkey());
    let recipient_token_account = test_ctx.create_associated_token_account(&mut context, &test_ctx.platform_admin, &test_ctx.mint.pubkey());

    // Add payment to escrow
    let escrow_token_account_balance = escrow_token_account.amount;
    assert_eq!(escrow_token_account_balance, payment_amount);

    let transaction = Transaction::new_signed_with_payer(
        &[
            process_payment(
                &test_ctx.program_test,
                test_ctx.user.pubkey(),
                payment_amount,
                user_token_account,
                escrow_token_account,
                recipient_token_account,
            ),
        ],
        Some(&context.payer.pubkey()),
        &[&context.payer, &test_ctx.user],
        context.last_blockhash,
    );

    context.banks_client.process_transaction(transaction).unwrap();
    let updated_balance = context.banks_client
        .get_account_data(&recipient_token_account)
        .unwrap()
        .amount;

    assert_eq!(updated_balance, payment_amount);
}

#[tokio::test]
async fn test_cancel_subscription() {
    let test_ctx = TestContext::new();
    test_ctx.create_accounts();

    let mut context = test_ctx.program_test.start_with_context().unwrap();

    let transaction = Transaction::new_signed_with_payer(
        &[
            cancel_subscription(
                &test_ctx.program_test,
                test_ctx.user.pubkey(),
                test_ctx.subscription_pda,
            ),
        ],
        Some(&context.payer.pubkey()),
        &[&context.payer, &test_ctx.user],
        context.last_blockhash,
    );

    context.banks_client.process_transaction(transaction).unwrap();
    let updated_subscription = context.banks_client
        .get_account_data(&test_ctx.subscription_pda)
        .unwrap();

    assert!(!updated_subscription.is_active);
}

#[tokio::test]
async fn test_update_fee_wallet() {
    let test_ctx = TestContext::new();
    test_ctx.create_accounts();

    let mut context = test_ctx.program_test.start_with_context().unwrap();

    let new_fee_wallet = Keypair::new().pubkey();

    let transaction = Transaction::new_signed_with_payer(
        &[
            update_fee_wallet(
                &test_ctx.program_test,
                test_ctx.platform_admin.pubkey(),
                new_fee_wallet,
            ),
        ],
        Some(&context.payer.pubkey()),
        &[&context.payer, &test_ctx.platform_admin],
        context.last_blockhash,
    );

    context.banks_client.process_transaction(transaction).unwrap();
    let updated_platform_config = context.banks_client
        .get_account_data(&test_ctx.fee_wallet)
        .unwrap();

    assert_eq!(updated_platform_config.fee_wallet, new_fee_wallet);
}

