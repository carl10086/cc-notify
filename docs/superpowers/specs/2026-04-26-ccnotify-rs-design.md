# CCNotify Rust 版设计文档

## 1. 项目概述

- **名称**: `ccnotify-rs`
- **目标**: 为 Claude Code 提供原生 macOS 通知，替代外部 `terminal-notifier`
- **用户场景**: 个人开发者自用
- **配置策略**: 零配置，开箱即用
- **核心依赖**: `notify-rust`、`serde`、`serde_json`

## 2. 设计约束

| 约束 | 决策 |
|------|------|
| 用户范围 | 仅个人使用，不考虑多用户或团队共享 |
| 配置方式 | 零配置文件，无环境变量，纯硬编码 |
| 终端应用 | 固定为 Ghostty（预留跳转功能，待 macOS 点击回调支持后启用） |
| 错误处理 | 静默失败，绝不阻塞 Claude Code 主流程 |
| 功能范围 | 支持 `Stop`、`Notification` 事件 |

## 3. 架构设计

### 3.1 模块结构

```
src/
├── main.rs      # 程序入口：参数解析、stdin 读取、错误兜底
├── event.rs     # 事件路由：根据 hook_event_name 和 reason 分发处理逻辑
└── notifier.rs  # 通知封装：notify-rust 调用、音频、body 构建
```

### 3.2 模块职责

| 模块 | 输入 | 输出 | 核心职责 |
|------|------|------|----------|
| `main` | `argv[1]` (event_name)、stdin JSON | 调用 `event::handle` | 解析输入，吞掉所有错误 |
| `event` | event_name、cwd、reason、permission_mode、notification_type | 调用 `notifier::notify` | 事件匹配、reason 映射 |
| `notifier` | NotificationContent | macOS 通知 | 发送通知 |

## 4. 数据结构

### 4.1 HookPayload

```rust
#[derive(Debug, serde::Deserialize)]
struct HookPayload {
    session_id: String,
    transcript_path: String,
    cwd: String,
    permission_mode: String,           // "ask" | "allow"
    hook_event_name: String,           // "Stop" | "Notification" | ...
    reason: Option<String>,            // Stop 事件特有
    notification_type: Option<String>, // Notification 事件特有
    #[serde(default)]
    user_prompt: Option<String>,       // UserPromptSubmit 特有（当前版本忽略）
}
```

### 4.2 NotificationContent

```rust
struct NotificationContent {
    title: String,              // 项目名（从 cwd 提取）
    subtitle: String,           // 事件: 原因
    body: String,               // cwd 路径
    sound: Option<&'static str>, // 音效名称
}
```

## 5. 数据流

```
Claude Code 触发 Hook
    ↓
执行: ~/.cargo/bin/ccnotify-rs <event_name>
    ↓
main.rs 读取 stdin → 解析完整 JSON（含 reason 等字段）
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

## 6. 事件处理逻辑

### 6.1 事件 → 通知映射

| 事件 | 标题 | 副标题 | Body | 音效 |
|------|------|--------|------|------|
| `Stop` | 项目名 | `Stop: {reason}` | `cwd` | 默认 |
| `Notification` | 项目名 | `Notification: {notification_type}` | `cwd` | 见下表 |
| 未知事件 | 项目名 | `{hook_event_name}` | `cwd` | 无 |

### 6.2 Notification 子类型映射

| notification_type | 副标题 | 音效 |
|-------------------|--------|------|
| `"permission"` | `Notification: Permission Required` | Breeze |
| `"input"` | `Notification: Waiting for input` | 默认 |
| `"action"` | `Notification: Action Required` | 默认 |
| 其他 / None | `Notification: Notification` | 默认 |

### 6.3 权限模式展示

当 `permission_mode == "ask"` 时，在副标题前加 🔒 前缀：

```
🔒 Stop: Task completed
```

## 7. UI 优化

### 7.1 通知样式

```
┌─────────────────────────────────────┐
│ 📁 项目名                              │  ← summary（粗体）
│ Stop: Task completed                 │  ← subtitle
│ cwd: /path/to/project               │  ← body
└─────────────────────────────────────┘
```

### 7.2 音效策略

| 事件 | 音效 | 说明 |
|------|------|------|
| `Stop` | 默认音效 | 任务完成 |
| `Notification: permission` | Breeze | 需要权限时用不同音效区分 |
| `Notification: input` | 默认 | 等待输入 |
| `Notification: action` | 默认 | 需要操作 |
| 其他 | 默认 | - |

### 7.3 平台差异

- **macOS**：`Hint::Resident` 不生效（永久停留需 macOS App 实现）
- **macOS**：`Timeout` 由系统通知中心控制，`notify-rust` 设置不生效
- **macOS**：`wait_for_action()` 不支持（点击跳转功能暂未实现）

## 8. 错误处理与边界情况

| 场景 | 行为 |
|------|------|
| `reason` 为空或 null | 副标题只显示 `hook_event_name` |
| `cwd` 为空 | 使用 `"Unknown"` 作为项目名 |
| `notification_type` 为空 | 使用 `"Notification"` 作为默认值 |
| JSON 解析失败 | 静默退出，return 0 |
| 未知 `hook_event_name` | 正常显示 `hook_event_name`，静默处理 |
| 通知发送失败 | `.show().ok()` 静默忽略 |
| `permission_mode == "ask"` | 副标题前加 🔒 前缀 |

## 9. 构建与安装

### 9.1 本地构建

```bash
cargo build --release
# 二进制: target/release/ccnotify-rs
```

### 9.2 安装

```bash
# 方式一：cargo install（需要网络）
cargo install --path .

# 方式二：手动复制
mkdir -p ~/.cargo/bin
cp target/release/ccnotify-rs ~/.cargo/bin/
chmod +x ~/.cargo/bin/ccnotify-rs
```

### 9.3 Claude Code 配置

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

## 10. 验收标准

1. `cargo install --path .` 后，`~/.cargo/bin/ccnotify-rs` 可执行
2. 配置 Hook 后，`Stop` 事件触发 macOS 原生通知
3. 通知显示：标题为项目名，副标题为 `Stop: {reason}`，body 为 cwd
4. `Notification: permission` 事件使用 Breeze 音效
5. `permission_mode == "ask"` 时副标题前显示 🔒 前缀
6. 程序异常时（如 stdin 为空、JSON 错误）不影响 Claude Code 正常使用
7. 零配置文件，首次使用仅需编译安装 + Hook 配置
8. 不依赖 `terminal-notifier` 或任何外部工具
