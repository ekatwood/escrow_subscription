import functions_framework
import requests
from flask import jsonify
import base64
import struct

# Constants for Raydium SOL/USDC pool
SOLANA_RPC = "https://api.mainnet-beta.solana.com"
RAYDIUM_POOL_ACCOUNT = "8HoQnePLqPj4M7PUDzfw8e3YMdPZ9oZdtPo9f5kPQCVe"  # SOL/USDC AMM ID

# We're fetching the pool state account which holds the token balances
def get_token_reserves():
    payload = {
        "jsonrpc": "2.0",
        "id": 1,
        "method": "getAccountInfo",
        "params": [
            RAYDIUM_POOL_ACCOUNT,
            {"encoding": "base64"}
        ]
    }
    res = requests.post(SOLANA_RPC, json=payload, timeout=10)
    account_data = res.json()["result"]["value"]["data"][0]
    raw_data = base64.b64decode(account_data)

    # Raydium AMM layout: token balances start at byte offset 64 and 72 (u64 each)
    base_token_amount = struct.unpack_from("<Q", raw_data, 64)[0]  # e.g., SOL
    quote_token_amount = struct.unpack_from("<Q", raw_data, 72)[0]  # e.g., USDC

    return base_token_amount, quote_token_amount

@functions_framework.http
def get_sol_for_usdc_fee(request):
    try:
        base_amt, quote_amt = get_token_reserves()

        if base_amt == 0:
            raise ValueError("Invalid pool state: zero base token amount")

        price_per_sol = quote_amt / base_amt / (10**6)  # USDC has 6 decimals
        sol_needed = 0.10 / price_per_sol

        return jsonify({
            "sol_needed": round(sol_needed, 8),
            "sol_price_usdc": round(price_per_sol, 4),
            "source": "raydium"
        })
    except Exception as e:
        return jsonify({"error": str(e)}), 500
