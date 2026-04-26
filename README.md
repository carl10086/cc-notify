# ccnotify-rs

Claude Code 原生 macOS 通知工具

## 项目概述

一个 Rust 编写的 Claude Code Hook 通知工具，为 Claude Code 提供原生 macOS 通知，
替代外部 terminal-notifier。零配置，开箱即用。

## 核心功能

- **Stop 事件通知** — 显示任务停止原因
- **Notification 事件通知** — 支持 permission/input/action 子类型
- **🔒 权限前缀** — permission_mode == "ask" 时显示锁图标
- **音效区分** — permission 事件使用 Breeze 音效

## 目录结构

```
src/
├── main.rs      # 程序入口，CLI 参数解析和 stdin JSON 解析
├── event.rs     # 事件路由，根据事件类型构建通知内容
└── notifier.rs  # 通知发送封装，调用 notify-rust
```

## 技术栈

- Rust 2021 Edition
- notify-rust — macOS 原生通知
- serde/serde_json — JSON 解析

## 通知效果示例

```
┌─────────────────────────────────────┐
│ 📁 my-project                       │  ← 标题（项目名）
│ Stop: Task completed                │  ← 副标题
│ cwd: /path/to/project              │  ← 正文
└─────────────────────────────────────┘

┌─────────────────────────────────────┐
│ 📁 my-project                       │
│ 🔒 Notification: Permission Required│
│ cwd: /path/to/project              │
└─────────────────────────────────────┘
```

## 事件类型说明

| 事件          | 副标题格式                        | 音效    |
|---------------|----------------------------------|---------|
| Stop          | Stop: {reason}                   | 默认    |
| Notification  | Notification: {子类型}            | 默认    |
| Notification  | Notification: Permission Required | Breeze |
| 未知事件      | {hook_event_name}               | 无      |

## 构建与安装

```bash
# 编译
cargo build --release

# 安装到 ~/.cargo/bin/
mkdir -p ~/.cargo/bin
cp target/release/ccnotify-rs ~/.cargo/bin/
chmod +x ~/.cargo/bin/ccnotify-rs

# 测试通知
~/.cargo/bin/ccnotify-rs test
```

## Claude Code 配置

在 `~/.claude/settings.json` 中添加：

```json
{
  "hooks": {
    "Stop": [{
      "hooks": [{
        "type": "command",
        "command": "~/.cargo/bin/ccnotify-rs Stop"
      }]
    }],
    "Notification": [{
      "hooks": [{
        "type": "command",
        "command": "~/.cargo/bin/ccnotify-rs Notification"
      }]
    }]
  }
}
```

## 测试

```bash
cargo test
```

## 架构设计

```
Claude Code 触发 Hook
    ↓
main.rs 读取 stdin JSON → 解析 HookPayload
    ↓
event::handle(event_name, cwd, reason, permission_mode, notification_type)
    ↓
构建 NotificationContent
    ↓
notifier::notify(content)
    ↓
notify-rust 发送通知（带音效）
    ↓
进程立即退出（通知由系统异步展示）
```
