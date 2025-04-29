# set up Store your SendGrid API Key securely as an environment variable in firebase
#SENDGRID_API_KEY="your-secret-api-key"
#FROM_EMAIL="receipts@yourdomain.com"

# request json format:
# {
#   "email": "user@example.com",
#   "wallet": "ABC123...XYZ",
#   "usdc_amount": 1000000,
#   "tx_signature": "4hvTz...Ek9h"
# }


import functions_framework
import os
from flask import request, jsonify
from sendgrid import SendGridAPIClient
from sendgrid.helpers.mail import Mail

# Load env vars
SENDGRID_API_KEY = os.environ.get("SENDGRID_API_KEY")
FROM_EMAIL = os.environ.get("FROM_EMAIL") or "receipts@yourdomain.com"

EXPLORER_URL = "https://solscan.io/tx/"  # Adjust for devnet if needed

@functions_framework.http
def send_receipt_email(req):
    try:
        data = req.get_json()
        email = data["email"]
        wallet = data["wallet"]
        usdc_amount = data["usdc_amount"]  # In smallest unit (e.g., 1000000 = 1 USDC)
        tx_signature = data["tx_signature"]

        amount_display = f"{usdc_amount / 1_000_000:.2f} USDC"
        explorer_link = f"{EXPLORER_URL}{tx_signature}"

        subject = "Your Subscription Payment Receipt"
        html_content = f"""
            <p>Hello,</p>
            <p>Thank you for your payment of <strong>{amount_display}</strong>.</p>
            <p>Wallet: <code>{wallet}</code></p>
            <p>Transaction: <a href="{explorer_link}" target="_blank">{tx_signature}</a></p>
            <br>
            <p>— The Team</p>
        """

        message = Mail(
            from_email=FROM_EMAIL,
            to_emails=email,
            subject=subject,
            html_content=html_content
        )

        sg = SendGridAPIClient(SENDGRID_API_KEY)
        response = sg.send(message)

        if 200 <= response.status_code < 300:
            return jsonify({"status": "sent"}), 200
        else:
            return jsonify({"error": "Failed to send email"}), 500

    except Exception as e:
        return jsonify({"error": str(e)}), 500
