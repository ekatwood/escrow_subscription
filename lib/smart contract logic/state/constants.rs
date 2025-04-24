use anchor_lang::prelude::*;

pub const DEFAULT_FEE: u64 = 10_000_000; // Default fee of $0.10 in smallest USDC unit (10**6 for 6 decimals)
pub const DECIMALS: u8 = 6; // USDC has 6 decimals
pub const FEE_BPS: u64 = 100; // Basis points for fee calculation (i.e., 0.10%)

pub const PROGRAM_ID: &str = "YourProgramIdHere"; // Replace with the actual Program ID when deploying
