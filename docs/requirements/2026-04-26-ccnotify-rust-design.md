# CCNotify Rust 版需求文档

## 1. 项目概述

**项目名称**: ccnotify-rs

**项目目标**: 用 Rust 重写 CCNotify，实现 Claude Code 的 Hook 事件通知功能，不依赖外部 `terminal-notifier` 工具。

**核心依赖**:
- `notify-rust` - 跨平台通知库，直接调用 macOS Notification Center API

---

## 2. 功能需求

### 2.1 支持的 Hook 事件

| 事件 | 触发时机 | 处理逻辑 |
|------|----------|----------|
| `UserPromptSubmit` | 用户提交 prompt 时 | 记录 session 开始时间 |
| `Stop` | Claude 停止时 | 发送"任务完成"通知 |
| `Notification` | Claude 发出通知时 | 检测类型，发送对应通知 |

### 2.2 通知类型

| 场景 | 通知标题 | 通知内容 |
|------|----------|----------|
| 任务完成 | 项目目录名 | "Task completed" |
| 等待输入 | 项目目录名 | "Waiting for input" |
| 需要权限 | 项目目录名 | "Permission Required" |
| 需要操作 | 项目目录名 | "Action Required" |

### 2.3 点击通知跳转

通知点击后，通过 AppleScript 跳转到 Ghostty 中对应项目的终端窗口。

**实现方式**: 使用 `osascript` 执行 AppleScript 命令，根据 `cwd` 查找并聚焦对应终端。

```bash
osascript -e '
tell application "Ghostty"
    set matches to every terminal whose working directory contains "/path/to/project"
    if (count of matches) > 0 then
        focus item 1 of matches
        activate window 1
    end if
end tell
'
```

---

## 3. Claude Hook 集成方式

### 3.1 配置文件

在 `~/.claude/settings.json` 中添加:

```json
{
  "hooks": {
    "UserPromptSubmit": [{
      "hooks": [{
        "type": "command",
        "command": "~/.cargo/bin/ccnotify-rs UserPromptSubmit"
      }]
    }],
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

### 3.2 数据传递方式

Claude Code 通过 **stdin** 传递 JSON 数据:

```json
{
  "session_id": "abc123",
  "transcript_path": "~/.claude/projects/.../transcript.jsonl",
  "cwd": "/path/to/project",
  "hook_event_name": "Stop"
}
```

### 3.3 交互流程

```
Claude Code 事件触发
    ↓
启动 ccnotify-rs 进程，传入 event_name
    ↓
从 stdin 读取 JSON 数据
    ↓
根据 event_name 处理事件，发送通知
    ↓
进程退出
```

---

## 4. 技术方案

### 4.1 核心依赖

| 依赖 | 用途 |
|------|------|
| `notify-rust` | 发送 macOS 原生通知 |
| `serde` / `serde_json` | 解析 Hook 传递的 JSON 数据 |

### 4.2 架构设计

```
src/
├── main.rs      # 程序入口，参数解析，stdin 读取
├── event.rs     # Hook 事件处理逻辑
└── notifier.rs  # 通知发送封装，AppleScript 跳转
```

### 4.3 通知发送

使用 `notify-rust` 库:

```rust
use notify_rust::Notification;

Notification::new()
    .title("项目名")
    .subtitle("Task completed")
    .show();
```

### 4.4 点击跳转

通过 `std::process::Command` 调用 `osascript` 执行 AppleScript:

```rust
fn jump_to_ghostty_terminal(cwd: &str) {
    let script = format!(
        r#"tell application "Ghostty"
            set matches to every terminal whose working directory contains "{}"
            if (count of matches) > 0 then
                focus item 1 of matches
                activate window 1
            end if
        end tell"#,
        cwd
    );

    std::process::Command::new("osascript")
        .args(["-e", &script])
        .spawn()
        .ok();
}
```

---

## 5. 验收标准

1. 可执行文件安装到 `~/.cargo/bin/ccnotify-rs`
2. 配置 Hook 后，`Stop` 事件能触发 macOS 通知
3. 不依赖 `terminal-notifier` 或任何外部工具
4. 程序异常时不影响 Claude Code 正常使用
5. 点击通知后能跳转到 Ghostty 对应项目的终端窗口
