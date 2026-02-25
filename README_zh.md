<div align="center">
  <a href="./README.md">English</a> | <strong>简体中文</strong>
</div>

<br>

<h1 align="center">Linear-Lark-Bridge 🌉</h1>

<p align="center">
  基于 Rust 构建的高性能中间件，将 Linear 的动态无缝同步至飞书。
  <br>
  极速响应、精确触达、支持富文本链接预览。
</p>

<p align="center">
  <a href="https://github.com/your-username/Linear-Lark-Bridge/actions"><img src="https://github.com/your-username/Linear-Lark-Bridge/actions/workflows/ci.yml/badge.svg" alt="CI Status"></a>
  <img src="https://img.shields.io/badge/Rust-1.75+-orange.svg" alt="Rust Version">
  <img src="https://img.shields.io/badge/部署-Railway-black.svg" alt="Railway">
  <img src="https://img.shields.io/badge/License-MIT-blue.svg" alt="License">
</p>

<hr>

## ✨ 核心特性

- ⚡️ **实时同步 (Phase 1)**：基于 Webhook 与 HMAC-SHA256 严谨验签，毫秒级将 Linear Issue 动态转化为飞书群 Interactive Card。
- 🎯 **精准私聊 (Phase 2)**：自动提取 payload 中的 `assignee.email`，免除手动维护 ID 映射表，直接调用飞书 Bot API 触达责任人。
- 🔗 **链接预览 (Phase 3)**：原生支持飞书 Challenge 握手验证机制。在飞书中发送 `linear.app` 链接，即刻展开富文本上下文卡片。
- 🚀 **极速部署**：内置 Docker 多阶段构建脚本，完美适配 Railway 等云原生 PaaS 平台，一键上线。
- 🔮 **面向未来**：底层架构已实现事件驱动解耦。未来的愿景是将其重构为 **Lark 统一集成中心 (Lark Integration Hub)**，接入 GitHub、Telegram 等多源生态。

## ⚙️ 环境变量配置

在启动项目或部署前，请确保以下环境变量已正确注入：

| 环境变量名 | 关联平台 | 必填 | 作用描述 |
| :--- | :---: | :---: | :--- |
| `LINEAR_WEBHOOK_SECRET` | Linear | ✅ | 用于验证 Webhook 来源的 HMAC 签名，防止伪造请求 |
| `LINEAR_API_KEY` | Linear | ✅ | 用于在处理飞书链接预览时，向 Linear 请求 Issue 详情 |
| `LARK_APP_ID` | 飞书 | ✅ | 飞书企业自建应用的 ID，用于获取 Tenant Access Token |
| `LARK_APP_SECRET` | 飞书 | ✅ | 飞书企业自建应用的密钥 |
| `LARK_VERIFICATION_TOKEN` | 飞书 | ✅ | 用于飞书事件回调的 Challenge 握手验证 |
| `PORT` | System | ❌ | Axum 监听端口 (默认 3000，云平台通常会自动分配) |

## 🛠️ 本地开发与调试

1. **环境准备：**
   ```bash
   git clone [https://github.com/your-username/Linear-Lark-Bridge.git](https://github.com/your-username/Linear-Lark-Bridge.git)
   cd Linear-Lark-Bridge
   # 复制并配置你的独立测试环境密钥（已配置 .gitignore 忽略）
   cp .env.example .env
2. **内网穿透 (ngrok)：**
    使用 ngrok http 3000 将本地端口暴露给外网，供 Linear 和飞书回调使用。

    ⚠️ 网络防坑指南： 如果你在本地使用 Surge (增强模式/Fake IP)，测试 Railway 域名或 ngrok 时可能会触发 198.18.x.x 路由冲突。请为相关域名配置直连 (Direct) 规则。

3. **代码门禁 (Pre-commit)：**
    项目使用 prek 保证提交质量，拦截格式与警告。

    ```bash
    cargo install prek
    prek install
    ```
## 📦 部署指南
    本项目推荐使用 Railway 进行部署。连接你的 GitHub 仓库后，Railway 会自动读取根目录的 Dockerfile 进行多阶段构建，全程无需人工干预。

## 📄 开源协议
MIT