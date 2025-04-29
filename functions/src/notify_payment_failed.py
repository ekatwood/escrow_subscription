# example request POST
# POST /notify_payment_failed
# Content-Type: application/json
#
# {
#   "wallet": "3x4DpJxA9nqSk9fYb9Fq5ZpQRZB..."
# }

# Required Firestore Fields (subscriptions/{pubkey})
# {
#   "email": "user@example.com",
#   ...
# }



import os
import functions_framework
from flask import request, jsonify
from google.cloud import firestore
from sendgrid import SendGridAPIClient
from sendgrid.helpers.mail import Mail

# Load environment variables
SENDGRID_API_KEY = os.environ.get("SENDGRID_API_KEY")
FROM_EMAIL = os.environ.get("FROM_EMAIL") or "receipts@yourdomain.com"

# Initialize Firestore
db = firestore.Client()

def send_failure_email(email: str, wallet: str):
    subject = "🚫 Subscription Payment Failed"
    html = f"""
        <p>Hello,</p>
        <p>Your recent subscription payment failed due to insufficient SOL in your wallet <code>{wallet}</code>.</p>
        <p>This small amount is required to cover Solana network fees (about $0.10 worth of SOL).</p>
        <p>Please top up your SOL balance and try again.</p>
        <br>
        <p>— The Team</p>
    """
    message = Mail(from_email=FROM_EMAIL, to_emails=email, subject=subject, html_content=html)
    SendGridAPIClient(SENDGRID_API_KEY).send(message)

@functions_framework.http
def notify_payment_failed(request):
    try:
        data = request.get_json(silent=True)

        wallet = data.get("wallet")
        if not wallet:
            return jsonify({"error": "Missing 'wallet'"}), 400

        doc = db.collection("subscriptions").document(wallet).get()
        if not doc.exists:
            return jsonify({"error": "Subscription not found"}), 404

        email = doc.to_dict().get("email")
        if not email:
            return jsonify({"error": "Email not set for this subscription"}), 400

        send_failure_email(email, wallet)
        return jsonify({"status": "email sent", "wallet": wallet}), 200

    except Exception as e:
        return jsonify({"error": str(e)}), 500
