// how to run:
// solana airdrop 2 <YourDevnetWalletAddress>
// anchor build
// anchor deploy --provider.cluster devnet
// ts-node devnet_script.ts

import { Connection, Keypair, PublicKey, Transaction, SystemProgram } from '@solana/web3.js';
import * as anchor from '@project-serum/anchor';
import { Token, TOKEN_PROGRAM_ID } from '@solana/spl-token';
import { Program, Idl, AnchorProvider } from '@project-serum/anchor';

// Set up the connection to Devnet
const connection = new Connection('https://api.devnet.solana.com', 'confirmed');
const payer = Keypair.generate(); // Payer's keypair (use Phantom Wallet in production)

// Set up Anchor Provider
const provider = new AnchorProvider(connection, new anchor.Wallet(payer), {
  commitment: 'confirmed',
  preflightCommitment: 'processed',
});

// Set up the IDL (Interface Definition Language) for your program
const idl: Idl = require('./target/idl/your_program.json'); // Replace with the correct path
const programID = new PublicKey('YourProgramPublicKey'); // Replace with your program's public key
const program = new Program(idl, programID, provider);

// Initialize or load the subscription account
const initializeSubscription = async () => {
  // Define the subscription account and fee wallet public keys
  const subscriptionAccount = Keypair.generate();
  const feeWallet = new PublicKey('YourFeeWalletPublicKey'); // Replace with actual fee wallet

  // Call the 'init_subscription' function from the program
  await program.rpc.initSubscription(feeWallet, {
    accounts: {
      subscription: subscriptionAccount.publicKey,
      user: payer.publicKey,
      feeWallet: feeWallet,
      systemProgram: SystemProgram.programId,
    },
    signers: [subscriptionAccount],
  });

  console.log(`Subscription initialized! Subscription account: ${subscriptionAccount.publicKey.toBase58()}`);
};

// Process a payment
const processPayment = async () => {
  const subscriptionAccount = Keypair.generate(); // Ensure this corresponds to an existing subscription account
  const escrowTokenAccount = Keypair.generate(); // This should have USDC in it
  const recipientTokenAccount = Keypair.generate(); // Recipient's token account (where USDC will be sent)

  const amountToPay = 1000000; // Example amount in 6 decimals (1 USDC)

  await program.rpc.processPayment(amountToPay, {
    accounts: {
      subscription: subscriptionAccount.publicKey,
      user: payer.publicKey,
      escrowTokenAccount: escrowTokenAccount.publicKey,
      recipientTokenAccount: recipientTokenAccount.publicKey,
      tokenProgram: TOKEN_PROGRAM_ID,
    },
    signers: [payer],
  });

  console.log(`Payment processed! Sent ${amountToPay / 1000000} USDC to the recipient.`);
};

// Set the price using the mock price oracle
const setPrice = async () => {
  const priceOracleAccount = Keypair.generate();
  const newPrice = 50000000; // 50 USDC per 1 SOL (in 6 decimal format)

  await program.rpc.setPrice(newPrice, {
    accounts: {
      priceOracleAccount: priceOracleAccount.publicKey,
      tokenProgram: TOKEN_PROGRAM_ID,
    },
    signers: [payer],
  });

  console.log(`Mock price set to ${newPrice / 1000000} USDC for 1 SOL`);
};

// Main function to deploy and interact with the contract
const main = async () => {
  // Ensure that you have enough SOL for the transactions
  const balance = await connection.getBalance(payer.publicKey);
  console.log(`Current balance: ${balance / 1000000000} SOL`);

  // Initialize the subscription
  await initializeSubscription();

  // Set the price for the mock oracle
  await setPrice();

  // Process a payment
  await processPayment();
};

// Run the main function
main()
  .then(() => console.log('Devnet script executed successfully!'))
  .catch((err) => console.error('Error executing devnet script:', err));
