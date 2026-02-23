
---

# 🌉 Lark-Linear-Integration

A high-performance, type-safe middleware service written in **Rust** that integrates [Linear](https://linear.app/) with [Lark / Feishu](https://larksuite.com/).

This bridge acts as a bidirectional webhook gateway, transforming Linear's raw JSON payloads into beautiful, interactive Lark cards, and handling Lark's event callbacks for rich link unfurling.

## ✨ Features

* **📢 Group Notifications**: Automatically pushes interactive Lark cards to a designated group chat when Linear Issues are created or updated. Cards are color-coded based on priority (Urgent = Red, High = Orange, etc.).
* **👤 Direct Message (DM) on Assign**: Automatically sends a private DM to a team member when a Linear issue is assigned to them. *(Note: This works by matching the assignee's Linear email with their Lark account email—no manual ID mapping required!)*
* **🔗 Link Previews (Unfurling)**: When a user pastes a `linear.app` link in a Lark chat, the bridge automatically catches the event, queries the Linear API, and unfurls the link into a detailed summary card.
* **🛡️ Secure by Default**: Implements strict HMAC-SHA256 signature verification for Linear webhooks and natively handles Lark's Challenge verification protocols.

## 🛠 Tech Stack

* **Language**: Rust (2021 Edition)
* **Web Framework**: `axum` + `tokio` (Async runtime)
* **HTTP Client**: `reqwest`
* **Serialization**: `serde` + `serde_json`
* **Cryptography**: `hmac` + `sha2`

## ⚙️ Environment Variables

To run this service, you must configure the following environment variables. In local development, you can place these in a `.env` file. In production (e.g., Railway), add them to your service variables.

### Linear Configuration

| Variable | Description | Where to find it |
| --- | --- | --- |
| `LINEAR_WEBHOOK_SECRET` | Used to verify payloads coming from Linear. | Linear Workspace Settings -> Integrations -> Webhooks |
| `LINEAR_API_KEY` | Used to query issue details for Link Previews. | Linear Settings -> API -> Personal API keys |

### Lark / Feishu Configuration

| Variable | Description | Where to find it |
| --- | --- | --- |
| `LARK_WEBHOOK_URL` | The Custom Bot webhook URL for pushing group notifications. | Lark Group Settings -> Bots -> Add Custom Bot |
| `LARK_APP_ID` | Your Lark Custom App ID (Required for DMs and Previews). | Lark Developer Console -> Credentials |
| `LARK_APP_SECRET` | Your Lark Custom App Secret. | Lark Developer Console -> Credentials |
| `LARK_VERIFICATION_TOKEN` | Used to verify Event Challenge requests from Lark. | Lark Developer Console -> Event Subscriptions |

### Server Configuration

| Variable | Description | Default |
| --- | --- | --- |
| `PORT` | The port the Axum server listens on. | `3000` |

---

## 🚀 Deployment (Railway)

This project is optimized for [Railway](https://railway.app/) using a highly efficient multi-stage `Dockerfile` to keep the image size minimal and deployment times fast.

1. Fork or clone this repository to your GitHub.
2. Create a **New Project** in Railway and select **Deploy from GitHub repo**.
3. Go to the **Variables** tab in your Railway project and add all the required environment variables listed above.
4. Generate a public domain in the **Networking** tab (e.g., `lark-linear-integration-production.up.railway.app`).

### Post-Deployment Setup

* **In Linear**: Set your Webhook URL to `https://<YOUR_RAILWAY_DOMAIN>/webhook`.
* **In Lark Developer Console**:
* Enable **Bot** and **Link Preview** capabilities.
* Add permissions for `im:message` and `contact:user.id:readonly`.
* Set your Link Preview URL pattern to `linear.app` and the Callback URL to `https://<YOUR_RAILWAY_DOMAIN>/lark/event`.



---

## 💻 Local Development & Testing

To test the integration locally without spamming your production team channels, follow this isolated testing pattern:

1. **Create a Test Environment**:
* Create a private Lark group with just yourself and add a new Custom Bot.
* Create a new "Local Debug" webhook in Linear.


2. **Start a Local Tunnel**:
Run `ngrok` to expose your local server to the internet.
```bash
ngrok http 3000

```


3. **Update Local Variables**:
Create a `.env` file in the project root (ensure `.env` is in your `.gitignore`) and temporarily swap in your test credentials:
```env
LARK_WEBHOOK_URL="<YOUR_TEST_GROUP_BOT_URL>"
LINEAR_WEBHOOK_SECRET="<YOUR_TEST_LINEAR_WEBHOOK_SECRET>"
# App ID, App Secret, and API Key can remain the same as production

```


4. **Run the Server**:
```bash
cargo run

```


5. **Update Linear Webhook**:
Point your "Local Debug" Linear webhook to `https://<YOUR_NGROK_URL>/webhook`. Make a change in Linear and watch the logs locally!

---

## 📝 License

[MIT License](https://www.google.com/search?q=LICENSE)
