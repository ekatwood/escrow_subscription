# input:
# {
#     "user_pubkey": "...",
#     "subscription_pubkey": "...",
#     "subscription_signer": "...",
#     "escrow_token_account": "...",
#     "recipient_token_account": "...",
#     "monthly_usdc_amount": 1000000
#   }



import functions_framework
import base64
import requests
import os
from flask import jsonify, request
from solana.publickey import PublicKey
from solana.transaction import Transaction, TransactionInstruction, AccountMeta
from solana.rpc.api import Client
from solana.system_program import transfer, TransferParams
from solana.rpc.types import TxOpts
from spl.token.constants import TOKEN_PROGRAM_ID

# === CONFIG ===
RPC_ENDPOINT = "https://api.mainnet-beta.solana.com"
PROGRAM_ID = PublicKey("YourProgram111111111111111111111111111111111111")
FEE_WALLET = PublicKey("FeeWallet1111111111111111111111111111111111111")
USDC_MINT = PublicKey("YourUSDCMintPubkeyHere")
PLATFORM_AUTHORITY = PublicKey("PlatformPayer11111111111111111111111111111")  # Optional, for meta

client = Client(RPC_ENDPOINT)

# === HELPER ===
def fetch_sol_fee_amount():
    url = "https://REGION-PROJECT.cloudfunctions.net/get_sol_for_usdc_fee"
    res = requests.get(url)
    return float(res.json()["sol_needed"])

@functions_framework.http
def create_subscription_tx(req):
    try:
        body = req.get_json()
        user_pubkey = PublicKey(body["user_pubkey"])
        subscription_pubkey = PublicKey(body["subscription_pubkey"])
        subscription_signer = PublicKey(body["subscription_signer"])
        escrow_token_account = PublicKey(body["escrow_token_account"])
        recipient_token_account = PublicKey(body["recipient_token_account"])
        monthly_usdc_amount = int(body["monthly_usdc_amount"])  # in USDC's smallest unit

        sol_fee = fetch_sol_fee_amount()
        lamports_fee = int(sol_fee * 1_000_000_000)

        # === Instruction 1: Anchor - process payment ===
        ix1 = TransactionInstruction(
            program_id=PROGRAM_ID,
            accounts=[
                AccountMeta(pubkey=subscription_pubkey, is_signer=False, is_writable=True),
                AccountMeta(pubkey=subscription_signer, is_signer=False, is_writable=False),
                AccountMeta(pubkey=user_pubkey, is_signer=True, is_writable=False),
                AccountMeta(pubkey=escrow_token_account, is_signer=False, is_writable=True),
                AccountMeta(pubkey=recipient_token_account, is_signer=False, is_writable=True),
                AccountMeta(pubkey=USDC_MINT, is_signer=False, is_writable=False),
                AccountMeta(pubkey=TOKEN_PROGRAM_ID, is_signer=False, is_writable=False),
            ],
            data=bytes([0])  # This assumes "process_payment" is instruction 0, adjust if needed
        )

        # === Instruction 2: Transfer SOL to fee wallet ===
        ix2 = transfer(
            TransferParams(
                from_pubkey=user_pubkey,
                to_pubkey=FEE_WALLET,
                lamports=lamports_fee
            )
        )

        # === Create TX ===
        tx = Transaction()
        tx.add(ix1)
        tx.add(ix2)

        # === Get latest blockhash ===
        latest_blockhash = client.get_latest_blockhash()["result"]["value"]["blockhash"]
        tx.recent_blockhash = latest_blockhash
        tx.fee_payer = user_pubkey  # Phantom signs this

        # === Serialize ===
        serialized = base64.b64encode(tx.serialize_message()).decode("utf-8")
        return jsonify({
            "transaction": serialized,
            "message": f"Approve subscription and fee payment of ~{sol_fee:.6f} SOL",
        })

    except Exception as e:
        return jsonify({"error": str(e)}), 500
