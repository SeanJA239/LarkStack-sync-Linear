<div align="center">
  <strong>English</strong> | <a href="./README_zh.md">简体中文</a>
</div>

<br>

<h1 align="center">Linear-Lark-Bridge 🌉</h1>

<p align="center">
  A high-performance middleware syncing Linear issues to Lark via Rust.
  <br>
  Built with Axum & Tokio for zero-delay workspace integration.
</p>

<p align="center">
  <a href="https://github.com/your-username/Linear-Lark-Bridge/actions"><img src="https://github.com/your-username/Linear-Lark-Bridge/actions/workflows/ci.yml/badge.svg" alt="CI Status"></a>
  <img src="https://img.shields.io/badge/Rust-1.75+-orange.svg" alt="Rust Version">
  <img src="https://img.shields.io/badge/Deployment-Railway-black.svg" alt="Railway">
  <img src="https://img.shields.io/badge/License-MIT-blue.svg" alt="License">
</p>

<hr>

## ✨ Features

- ⚡️ **Real-time Sync**: Receives Linear Webhooks, verifies HMAC-SHA256 signatures, and pushes highly customized Interactive Cards to Lark groups in milliseconds.
- 🎯 **Direct Messaging**: Automatically extracts `assignee.email` from the payload to send direct messages via the Lark Bot API—no manual ID mapping required!
- 🔗 **Link Previews**: Natively supports Lark's Challenge handshake protocol. Pasting a Linear link in Lark instantly renders a rich context card.
- 🚀 **Cloud Native**: Ships with a multi-stage `Dockerfile` optimized for zero-config deployment on PaaS platforms like Railway.
- 🔮 **Future-Proof Architecture**: Designed with decoupled event-driven routing, paving the way for the ultimate **Lark Integration Hub** (supporting GitHub, Telegram, etc. in the future).

## ⚙️ Configuration

Ensure the following environment variables are set before running the application:

| Environment Variable | Platform | Required | Description |
| :--- | :---: | :---: | :--- |
| `LINEAR_WEBHOOK_SECRET` | Linear | ✅ | Used to verify the HMAC signature of incoming webhooks. |
| `LINEAR_API_KEY` | Linear | ✅ | Used to fetch issue details for Link Previews. |
| `LARK_APP_ID` | Lark | ✅ | Used to obtain the Tenant Access Token. |
| `LARK_APP_SECRET` | Lark | ✅ | Used in conjunction with App ID for authentication. |
| `LARK_VERIFICATION_TOKEN` | Lark | ✅ | Used to validate Lark's Event Callback (Challenge handshake). |
| `PORT` | System | ❌ | Axum listening port (Defaults to 3000). |

## 🛠️ Local Development

1. **Clone & Setup:**
   ```bash
   git clone [https://github.com/your-username/Linear-Lark-Bridge.git](https://github.com/your-username/Linear-Lark-Bridge.git)
   cd Linear-Lark-Bridge
   cp .env.example .env # Fill in your development secrets
2. **Network Routing (ngrok):**
    Use ngrok http 3000 to expose your local server.

    ⚠️ Note for Surge users: If using Enhanced Mode (Fake IP), accessing your Railway/ngrok domains locally might cause 198.18.x.x routing conflicts. Configure direct connections for these domains.

3. **Pre-commit Gatekeeper:**
    We enforce strict code quality using prek.

    ```bash
    cargo install prek
    prek install
    ```
## 📦 Deployment
    This project is optimized for Railway. Simply connect your GitHub repository to Railway, and the platform will automatically build and deploy the app using the provided multi-stage Dockerfile.

## 📄 License
    MIT