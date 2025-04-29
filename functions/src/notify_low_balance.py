# env variables to store in firebase
# SENDGRID_API_KEY=your-sendgrid-key
# FROM_EMAIL=receipts@yourdomain.com
# SOLANA_RPC=https://api.mainnet-beta.solana.com  # or devnet/testnet

# firestore field required
# subscriptions/{pubkey}
# {
#   "email": "user@example.com",
#   "monthly_amount": 1000000,
#   "escrow_address": "F13fG...escrow"
# }

# schedule for once a day
# gcloud scheduler jobs create http notify-low-balance \
#   --schedule="0 9 * * *" \
#   --uri="https://REGION-PROJECT.cloudfunctions.net/notify_low_balance" \
#   --http-method=GET \
#   --time-zone="UTC"


import os
import functions_framework
from flask import jsonify
from sendgrid import SendGridAPIClient
from sendgrid.helpers.mail import Mail
from google.cloud import firestore
from solana.rpc.api import Client
from solana.publickey import PublicKey

# Environment
SENDGRID_API_KEY = os.environ.get("SENDGRID_API_KEY")
FROM_EMAIL = os.environ.get("FROM_EMAIL") or "receipts@yourdomain.com"
SOLANA_RPC = os.environ.get("SOLANA_RPC") or "https://api.mainnet-beta.solana.com"

# Clients
db = firestore.Client()
solana_client = Client(SOLANA_RPC)

def send_low_balance_email(email: str, wallet: str):
    subject = "⚠️ Low Subscription Balance Alert"
    html = f"""
        <p>Hello,</p>
        <p>Your subscription escrow balance for wallet <code>{wallet}</code> is running low and may not cover your next billing cycle.</p>
        <p>Please top up your escrow to avoid service interruption.</p>
        <br><p>— The Team</p>
    """
    message = Mail(from_email=FROM_EMAIL, to_emails=email, subject=subject, html_content=html)
    SendGridAPIClient(SENDGRID_API_KEY).send(message)

@functions_framework.http
def notify_low_balance(_):
    subscriptions = db.collection("subscriptions").stream()

    notified_count = 0

    for doc in subscriptions:
        sub = doc.to_dict()
        wallet = doc.id
        email = sub.get("email")
        escrow_address = sub.get("escrow_address")
        monthly_amount = sub.get("monthly_amount")

        if not all([email, escrow_address, monthly_amount]):
            continue

        try:
            escrow_pubkey = PublicKey(escrow_address)
            resp = solana_client.get_token_account_balance(escrow_pubkey)
            amount_raw = int(resp["result"]["value"]["amount"])

            if amount_raw < monthly_amount:
                send_low_balance_email(email, wallet)
                notified_count += 1

        except Exception as e:
            print(f"Error checking {wallet}: {str(e)}")
            continue

    return jsonify({"status": "completed", "notified": notified_count}), 200
