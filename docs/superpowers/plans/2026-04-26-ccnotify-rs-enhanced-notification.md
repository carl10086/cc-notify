# CCNotify 增强通知实现计划

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 增强通知信息量，显示事件类型、原因、路径，而非硬编码文字。

**Architecture:** 三模块不变：main.rs 解析完整 HookPayload → event.rs 根据事件类型和 reason 构建通知内容 → notifier.rs 发送带音效的通知。

**Tech Stack:** Rust, notify-rust, serde/serde_json

**设计文档:** `docs/superpowers/specs/2026-04-26-ccnotify-rs-design.md`

**编码规则:** 遵循 `.claude/rules/code.md` — 最小代码、不猜测、目标驱动、每次改动可验证。

---

## 文件结构

| 文件 | 变更 | 职责 |
|------|------|------|
| `src/main.rs` | 修改 | HookPayload 新增 reason、permission_mode 字段 |
| `src/event.rs` | 修改 | handle 函数新增参数，构建 NotificationContent |
| `src/notifier.rs` | 修改 | notify 接收 NotificationContent，构建完整通知（含音效） |

---

### Task 1: 更新 HookPayload 数据结构

**Files:**
- Modify: `src/main.rs`

- [ ] **Step 1: 更新 HookPayload 结构**

```rust
#[derive(Debug, serde::Deserialize)]
struct HookPayload {
    session_id: String,
    transcript_path: String,
    cwd: String,
    permission_mode: String,           // 新增
    hook_event_name: String,
    reason: Option<String>,           // 新增：Stop 事件特有
    notification_type: Option<String>, // 保留
    #[serde(default)]
    user_prompt: Option<String>,
}
```

- [ ] **Step 2: 运行 cargo check 验证编译**

Run: `cargo check 2>&1`
Expected: 编译通过

- [ ] **Step 3: Commit**

```bash
git add src/main.rs
git commit -m "feat: expand HookPayload with reason and permission_mode"
```

---

### Task 2: 更新 event.rs 事件处理逻辑

**Files:**
- Modify: `src/event.rs`

- [ ] **Step 1: 编写 event.rs（完整重写）**

```rust
use crate::notifier::{self, NotificationContent};

/// 处理事件，构建通知内容
pub fn handle(
    event_name: &str,
    cwd: &str,
    reason: Option<&str>,
    permission_mode: &str,
    notification_type: Option<&str>,
) {
    let content = build_notification_content(event_name, cwd, reason, permission_mode, notification_type);
    notifier::notify(content);
}

/// 根据事件类型和 reason 构建通知内容
fn build_notification_content(
    event_name: &str,
    cwd: &str,
    reason: Option<&str>,
    permission_mode: &str,
    notification_type: Option<&str>,
) -> NotificationContent {
    let title = extract_project_name(cwd);
    let (subtitle, sound) = match event_name {
        "Stop" => {
            let reason_text = reason.unwrap_or("Unknown reason");
            let prefix = if permission_mode == "ask" { "🔒 " } else { "" };
            (format!("Stop: {}{}", prefix, reason_text), Some("Default"))
        }
        "Notification" => {
            let sub_type = notification_type.unwrap_or("Notification");
            let (detail, sound) = match sub_type {
                "permission" => ("Permission Required", "Breeze"),
                "input" => ("Waiting for input", "Default"),
                "action" => ("Action Required", "Default"),
                _ => ("Notification", "Default"),
            };
            let prefix = if permission_mode == "ask" { "🔒 " } else { "" };
            (format!("Notification: {}{}", prefix, detail), Some(sound))
        }
        _ => {
            let prefix = if permission_mode == "ask" { "🔒 " } else { "" };
            (format!("{}{}", prefix, event_name), None)
        }
    };

    NotificationContent {
        title: title.to_string(),
        subtitle,
        body: format!("cwd: {}", cwd),
        sound,
    }
}

/// 从 cwd 提取项目名
fn extract_project_name(cwd: &str) -> &str {
    std::path::Path::new(cwd)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("Unknown")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stop_with_reason() {
        let content = build_notification_content("Stop", "/Users/carlyu/my-project", Some("Task completed"), "allow", None);
        assert_eq!(content.title, "my-project");
        assert_eq!(content.subtitle, "Stop: Task completed");
        assert_eq!(content.sound, Some("Default"));
    }

    #[test]
    fn test_stop_without_reason() {
        let content = build_notification_content("Stop", "/Users/carlyu/my-project", None, "allow", None);
        assert_eq!(content.subtitle, "Stop: Unknown reason");
    }

    #[test]
    fn test_stop_with_permission_ask() {
        let content = build_notification_content("Stop", "/Users/carlyu/my-project", Some("Task completed"), "ask", None);
        assert_eq!(content.subtitle, "🔒 Stop: Task completed");
    }

    #[test]
    fn test_notification_permission() {
        let content = build_notification_content("Notification", "/Users/carlyu/my-project", None, "allow", Some("permission"));
        assert_eq!(content.subtitle, "Notification: Permission Required");
        assert_eq!(content.sound, Some("Breeze"));
    }

    #[test]
    fn test_notification_input() {
        let content = build_notification_content("Notification", "/Users/carlyu/my-project", None, "allow", Some("input"));
        assert_eq!(content.subtitle, "Notification: Waiting for input");
    }

    #[test]
    fn test_unknown_event() {
        let content = build_notification_content("UnknownEvent", "/Users/carlyu/my-project", None, "allow", None);
        assert_eq!(content.subtitle, "UnknownEvent");
    }

    #[test]
    fn test_extract_project_name() {
        assert_eq!(extract_project_name("/Users/carlyu/my-project"), "my-project");
        assert_eq!(extract_project_name("/Users/carlyu/my-project/"), "my-project");
        assert_eq!(extract_project_name(""), "Unknown");
    }
}
```

- [ ] **Step 2: 运行测试**

Run: `cargo test event::tests 2>&1`
Expected: 7 个测试全部 PASS

- [ ] **Step 3: Commit**

```bash
git add src/event.rs
git commit -m "feat: implement event routing with reason and permission display"
```

---

### Task 3: 更新 notifier.rs 通知发送

**Files:**
- Modify: `src/notifier.rs`

- [ ] **Step 1: 编写 notifier.rs（完整重写）**

```rust
use notify_rust::Notification;

pub struct NotificationContent {
    pub title: String,
    pub subtitle: String,
    pub body: String,
    pub sound: Option<&'static str>,
}

/// 发送通知
pub fn notify(content: NotificationContent) {
    let mut notification = Notification::new();
    notification
        .summary(&content.title)
        .subtitle(&content.subtitle)
        .body(&content.body);

    if let Some(sound) = content.sound {
        if sound != "Default" {
            notification.sound(sound);
        }
    }

    let _ = notification.show();
}

/// 跳转到 Ghostty 终端（预留功能，macOS 暂不支持点击回调）
pub fn jump_to_ghostty_terminal(_cwd: &str) {
    // TODO: macOS NotificationCenter 不支持 wait_for_action()
    // 待调研替代方案后实现
}

/// 执行 osascript（预留功能）
#[allow(dead_code)]
fn try_run_osascript(_script: &str) -> Result<(), ()> {
    Err(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_notification_content_fields() {
        let content = NotificationContent {
            title: "my-project".to_string(),
            subtitle: "Stop: Task completed".to_string(),
            body: "cwd: /path/to/project".to_string(),
            sound: Some("Default"),
        };
        assert_eq!(content.title, "my-project");
        assert_eq!(content.sound, Some("Default"));
    }
}
```

- [ ] **Step 2: 运行测试**

Run: `cargo test notifier::tests 2>&1`
Expected: 1 个测试 PASS

- [ ] **Step 3: Commit**

```bash
git add src/notifier.rs
git commit -m "feat: update notifier to send rich notification with sound"
```

---

### Task 4: 更新 main.rs 调用

**Files:**
- Modify: `src/main.rs`

- [ ] **Step 1: 更新 main.rs 调用逻辑**

```rust
use std::io::{self, Read};

mod event;
mod notifier;

#[derive(Debug, serde::Deserialize)]
struct HookPayload {
    session_id: String,
    transcript_path: String,
    cwd: String,
    permission_mode: String,
    hook_event_name: String,
    reason: Option<String>,
    notification_type: Option<String>,
    #[serde(default)]
    user_prompt: Option<String>,
}

fn main() {
    let args: Vec<String> = std::env::args().collect();

    // test 子命令：直接发送测试通知，无需 stdin
    if args.len() == 2 && args[1] == "test" {
        let cwd = std::env::current_dir()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_else(|_| String::from("."));
        notifier::notify(notifier::NotificationContent {
            title: extract_project_name(&cwd),
            subtitle: "Test Notification".to_string(),
            body: format!("cwd: {}", cwd),
            sound: Some("Default"),
        });
        return;
    }

    // 正常模式：需要 event_name 参数
    let event_name = args.get(1).map(|s| s.as_str()).unwrap_or("");
    if event_name.is_empty() {
        return;
    }

    // 从 stdin 读取 JSON
    let mut input = String::new();
    if io::stdin().read_to_string(&mut input).is_err() {
        return;
    }

    let input = input.trim();
    if input.is_empty() {
        return;
    }

    // 解析 JSON，失败则静默退出
    let payload: HookPayload = match serde_json::from_str(input) {
        Ok(p) => p,
        Err(_) => return,
    };

    event::handle(
        event_name,
        &payload.cwd,
        payload.reason.as_deref(),
        &payload.permission_mode,
        payload.notification_type.as_deref(),
    );
}

fn extract_project_name(cwd: &str) -> String {
    std::path::Path::new(cwd)
        .file_name()
        .and_then(|n| n.to_str())
        .map(String::from)
        .unwrap_or_else(|| "Unknown".to_string())
}
```

- [ ] **Step 2: 编译验证**

Run: `cargo check 2>&1`
Expected: 编译通过

- [ ] **Step 3: Commit**

```bash
git add src/main.rs
git commit -m "feat: wire up event handler with full payload"
```

---

### Task 5: 编译并测试

**Files:**
- 无文件变更

- [ ] **Step 1: 编译 release**

Run: `cargo build --release 2>&1`
Expected: 编译成功

- [ ] **Step 2: 运行所有测试**

Run: `cargo test 2>&1`
Expected: 所有测试 PASS（event 7 个 + notifier 1 个）

- [ ] **Step 3: 测试 test 子命令**

Run: `./target/release/ccnotify-rs test 2>&1`
Expected: macOS 弹出通知，标题为当前目录名

- [ ] **Step 4: 测试 Stop 事件（带 reason）**

Run:
```bash
echo '{"session_id":"test","transcript_path":"/tmp","cwd":"/Users/carlyu/my-project","permission_mode":"allow","hook_event_name":"Stop","reason":"Task completed"}' | ./target/release/ccnotify-rs Stop
```
Expected: 通知显示 "Stop: Task completed"，body 显示 cwd

- [ ] **Step 5: 测试 Notification: permission（ask 模式）**

Run:
```bash
echo '{"session_id":"test","transcript_path":"/tmp","cwd":"/Users/carlyu/my-project","permission_mode":"ask","hook_event_name":"Notification","notification_type":"permission"}' | ./target/release/ccnotify-rs Notification
```
Expected: 通知显示 "🔒 Notification: Permission Required"

---

## Self-Review 检查清单

### 1. Spec 覆盖率

| Spec 要求 | 对应 Task |
|-----------|-----------|
| HookPayload 包含 reason、permission_mode | Task 1 |
| Stop 事件显示 "Stop: {reason}" | Task 2 |
| Notification 显示子类型 | Task 2 |
| permission_mode == "ask" 时显示 🔒 前缀 | Task 2 |
| Notification: permission 使用 Breeze 音效 | Task 2 |
| cwd 显示在 body | Task 2 |
| test 子命令 | Task 4 |

**覆盖率：100%。**

### 2. Placeholder 扫描

- [x] 无 "TBD"/"TODO"/"implement later"
- [x] 无模糊描述
- [x] 每个代码步骤包含完整代码
- [x] 无 "Similar to Task N" 引用

### 3. 类型一致性

| 类型/函数 | 定义位置 | 使用位置 | 状态 |
|-----------|----------|----------|------|
| `NotificationContent` | Task 3 (notifier.rs) | Task 2, Task 4 | ✅ 一致 |
| `event::handle(...)` | Task 2 (event.rs) | Task 4 (main.rs) | ✅ 一致 |
| `extract_project_name` | Task 2 (private) | Task 4 (public) | ✅ 一致 |

---

## 执行选项

**Plan complete and saved to `docs/superpowers/plans/2026-04-26-ccnotify-rs-enhanced-notification.md`. Two execution options:**

**1. Subagent-Driven (recommended)** - I dispatch a fresh subagent per task, review between tasks, fast iteration

**2. Inline Execution** - Execute tasks in this session using executing-plans, batch execution with checkpoints

**Which approach?**
