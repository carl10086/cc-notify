# ccnotify-rs 实现计划

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 实现 ccnotify-rs，一个零配置的 Claude Code Hook 通知工具，支持 Stop/Notification 事件发送 macOS 原生通知，点击后跳转 Ghostty 终端。

**Architecture:** 三模块结构：main.rs 处理 CLI 参数和 stdin；event.rs 根据事件名路由到对应处理；notifier.rs 封装 notify-rust 通知发送和 AppleScript 终端跳转。主进程发送通知后通过 channel + recv_timeout 等待点击，超时 30 秒后退出。

**Tech Stack:** Rust, notify-rust, serde/serde_json

**设计文档:** `docs/superpowers/specs/2026-04-26-ccnotify-rs-design.md`

**编码规则:** 遵循 `.claude/rules/code.md` — 最小代码、不猜测、目标驱动、每次改动可验证。

---

## Refactor Review 修复摘要

基于 refactor skill 的审查，本 plan 已做以下优化：

| 问题 | 修复方案 |
|------|----------|
| 悬空指针风险（`&str` 移入线程） | `notify(cwd: &str)` → `notify(cwd: String)`，主调方传入所有权 |
| sleep 30 秒阻塞主线程 | 改为 `mpsc::channel + recv_timeout`：点击立即响应，未点击则 30 秒后超时退出 |
| notify-rust `.action()` API 误用 | 第二个参数改为 "Open"（按钮标签），非重复 action ID |
| `notify` 函数职责混杂 | 拆分为 `build_notification`、`send_notification`、`wait_for_click` 三个纯函数 |
| `UserPromptSubmit` 空分支（Speculative Generality） | 完全移除，减少无意义进程 fork |

---

## 文件结构

| 文件 | 职责 | 状态 |
|------|------|------|
| `Cargo.toml` | 依赖声明 | 已存在，需微调 |
| `src/main.rs` | 程序入口：参数解析（含 test 子命令）、stdin 读取、错误兜底 | 已存在，需重写 |
| `src/event.rs` | 事件路由：仅处理 Stop 和 Notification | 已存在，需重写 |
| `src/notifier.rs` | 通知封装：notify-rust 发送 + AppleScript 跳转（含 fallback） | 已存在，需重写 |

---

### Task 1: 更新 Cargo.toml 确认依赖

**Files:**
- Modify: `Cargo.toml`

当前 `Cargo.toml` 已包含 `notify-rust`、`serde`、`serde_json`。需要确认版本和添加 `edition`。

- [ ] **Step 1: 更新 Cargo.toml**

```toml
[package]
name = "ccnotify-rs"
version = "0.1.0"
edition = "2021"

[dependencies]
notify-rust = "4"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
```

- [ ] **Step 2: 验证文件正确**

Run: `cat Cargo.toml`
Expected: 输出与上面一致

- [ ] **Step 3: Commit**

```bash
git add Cargo.toml
git commit -m "chore: confirm dependencies"
```

---

### Task 2: 实现 notifier.rs（通知发送 + AppleScript 跳转）

**Files:**
- Create: `src/notifier.rs`

职责：发送 macOS 通知，点击 "Open" 按钮时通过 AppleScript 跳转到 Ghostty 终端。包含双重 fallback 策略。

**设计要点**（基于 refactor review 的修复）：
- `notify` 接收 `String` 而非 `&str`，避免线程间悬空指针
- 使用 `mpsc::channel + recv_timeout` 等待点击，替代 `sleep` 阻塞
- 修复 `.action()` API 误用：第二个参数为按钮标签 "Open"
- 拆分职责：`extract_project_name`、`build_notification`、`send_notification`、`wait_for_click`

- [ ] **Step 1: 编写 notifier.rs**

```rust
use notify_rust::Notification;
use notify_rust::NotificationHandle;
use std::path::Path;
use std::process::Command;
use std::sync::mpsc;
use std::time::Duration;

/// 从 cwd 提取项目名作为通知标题
fn extract_project_name(cwd: &str) -> &str {
    Path::new(cwd)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or(cwd)
}

/// 构建通知对象
fn build_notification<'a>(title: &'a str, subtitle: &'a str) -> Notification {
    Notification::new()
        .summary(title)
        .subtitle(subtitle)
        .action("default", "Open")
        .clone()
}

/// 发送通知并返回 handle
fn send_notification(notification: Notification) -> Option<NotificationHandle> {
    notification.show().ok()
}

/// 等待用户点击通知按钮，超时返回 None
/// 使用 channel + recv_timeout 实现：点击后立即响应，未点击则 30 秒后超时
fn wait_for_click(handle: NotificationHandle, timeout: Duration) -> Option<String> {
    let (tx, rx) = mpsc::channel();

    std::thread::spawn(move || {
        if let Ok(action) = handle.wait_for_action() {
            let _ = tx.send(action);
        }
    });

    rx.recv_timeout(timeout).ok()
}

/// 跳转到 Ghostty 终端（双重 fallback）
fn jump_to_ghostty_terminal(cwd: &str) {
    // 尝试 1: 精确匹配 working directory
    let script_precise = format!(
        r#"tell application "Ghostty"
            set matches to every terminal whose working directory contains "{}"
            if (count of matches) > 0 then
                focus item 1 of matches
                activate window 1
            end if
        end tell"#,
        cwd
    );

    if try_run_osascript(&script_precise).is_ok() {
        return;
    }

    // 尝试 2: 仅激活 Ghostty
    let _ = try_run_osascript(r#"tell application "Ghostty" to activate"#);
}

/// 执行 osascript，成功返回 Ok，失败返回 Err
fn try_run_osascript(script: &str) -> Result<(), ()> {
    Command::new("osascript")
        .args(["-e", script])
        .output()
        .map(|output| {
            if output.status.success() {
                Ok(())
            } else {
                Err(())
            }
        })
        .unwrap_or(Err(()))
}

/// 发送通知，等待用户点击 "Open" 按钮后跳转到 Ghostty
/// 使用 String 而非 &str，避免线程间悬空指针
pub fn notify(cwd: String, subtitle: &str) {
    let title = extract_project_name(&cwd);
    let notification = build_notification(title, subtitle);

    if let Some(handle) = send_notification(notification) {
        if let Some(action) = wait_for_click(handle, Duration::from_secs(30)) {
            if action == "default" {
                jump_to_ghostty_terminal(&cwd);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_project_name_from_valid_path() {
        assert_eq!(extract_project_name("/Users/carlyu/my-project"), "my-project");
    }

    #[test]
    fn test_extract_project_name_from_trailing_slash() {
        assert_eq!(extract_project_name("/Users/carlyu/my-project/"), "my-project");
    }

    #[test]
    fn test_extract_project_name_fallback() {
        assert_eq!(extract_project_name(""), "");
    }
}
```

- [ ] **Step 2: 运行单元测试**

Run: `cargo test --lib`
Expected: 3 个测试全部 PASS

- [ ] **Step 3: Commit**

```bash
git add src/notifier.rs
git commit -m "feat: implement notifier with notify-rust and AppleScript fallback"
```

---

### Task 3: 实现 event.rs（事件路由）

**Files:**
- Create: `src/event.rs`

职责：根据 event_name 和可选的 notification_type 确定通知副标题，调用 notifier。

**设计要点**：移除 `UserPromptSubmit` 空分支（refactor review: Speculative Generality），`cwd` 参数改为 `String` 传递所有权。

- [ ] **Step 1: 编写 event.rs**

```rust
use crate::notifier;

/// 处理事件，根据 event_name 发送对应通知
/// UserPromptSubmit 当前版本不处理（已从 Hook 配置中移除），减少无意义进程 fork
pub fn handle(event_name: &str, cwd: String, notification_type: Option<&str>) {
    match event_name {
        "Stop" => {
            notifier::notify(cwd, "Task completed");
        }
        "Notification" => {
            let subtitle = map_notification_type(notification_type);
            notifier::notify(cwd, subtitle);
        }
        _ => {
            // 未知事件，静默忽略
        }
    }
}

/// 将 notification_type 映射为通知副标题
fn map_notification_type(notification_type: Option<&str>) -> &'static str {
    match notification_type {
        Some("input") => "Waiting for input",
        Some("permission") => "Permission Required",
        Some("action") => "Action Required",
        _ => "Notification",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_map_notification_type_input() {
        assert_eq!(map_notification_type(Some("input")), "Waiting for input");
    }

    #[test]
    fn test_map_notification_type_permission() {
        assert_eq!(map_notification_type(Some("permission")), "Permission Required");
    }

    #[test]
    fn test_map_notification_type_action() {
        assert_eq!(map_notification_type(Some("action")), "Action Required");
    }

    #[test]
    fn test_map_notification_type_unknown() {
        assert_eq!(map_notification_type(Some("unknown")), "Notification");
    }

    #[test]
    fn test_map_notification_type_none() {
        assert_eq!(map_notification_type(None), "Notification");
    }
}
```

- [ ] **Step 2: 运行单元测试**

Run: `cargo test --lib`
Expected: 5 个测试全部 PASS（包含 Task 2 的 3 个测试）

- [ ] **Step 3: Commit**

```bash
git add src/event.rs
git commit -m "feat: implement event routing with notification type mapping"
```

---

### Task 4: 实现 main.rs（入口 + test 子命令）

**Files:**
- Create: `src/main.rs`

职责：解析 CLI 参数，支持 `test` 子命令（无需 stdin）；正常模式从 stdin 读取 JSON，解析后调用 event::handle。所有错误静默处理。

**设计要点**：`notifier::notify` 和 `event::handle` 调用时传入 `String` 所有权（配合 Task 2/3 的签名变更）。

- [ ] **Step 1: 编写 main.rs**

```rust
use std::io::{self, Read};

mod event;
mod notifier;

#[derive(Debug, serde::Deserialize)]
struct HookPayload {
    session_id: String,
    transcript_path: String,
    cwd: String,
    hook_event_name: String,
    notification_type: Option<String>,
}

fn main() {
    let args: Vec<String> = std::env::args().collect();

    // test 子命令：直接发送测试通知，无需 stdin
    if args.len() == 2 && args[1] == "test" {
        let cwd = std::env::current_dir()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_else(|_| String::from("."));
        notifier::notify(cwd, "Test Notification");
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

    event::handle(event_name, payload.cwd, payload.notification_type.as_deref());
}
```

- [ ] **Step 2: 编译验证**

Run: `cargo check`
Expected: 编译通过，无错误（可能有 warning）

- [ ] **Step 3: Commit**

```bash
git add src/main.rs
git commit -m "feat: implement main with stdin parsing and test subcommand"
```

---

### Task 5: 编译 Release 版本

**Files:**
- 无文件变更

- [ ] **Step 1: 构建 release**

Run: `cargo build --release`
Expected: 编译成功，生成 `target/release/ccnotify-rs`

- [ ] **Step 2: 验证二进制存在**

Run: `ls -lh target/release/ccnotify-rs`
Expected: 显示文件存在，大小约 1-3MB

- [ ] **Step 3: Commit（如有 Cargo.lock 更新）**

```bash
git add -A
git diff --cached --quiet || git commit -m "chore: update Cargo.lock after build"
```

---

### Task 6: 运行 Examples 验证功能

**Files:**
- 无文件变更

使用设计文档中的 examples 验证通知功能和点击跳转。

- [ ] **Step 1: 测试 test 子命令**

Run: `./target/release/ccnotify-rs test`
Expected: macOS 弹出通知，标题为当前目录名，副标题为 "Test Notification"
操作：点击通知上的 "Open" 按钮，观察 Ghostty 是否被激活

- [ ] **Step 2: 测试 Stop 事件**

Run:
```bash
echo '{"session_id":"test","transcript_path":"/tmp","cwd":"/Users/carlyu/my-project","hook_event_name":"Stop"}' | ./target/release/ccnotify-rs Stop
```
Expected: macOS 弹出通知，标题为 "my-project"，副标题为 "Task completed"
操作：点击 "Open" 按钮，观察 Ghostty 是否被激活或聚焦

- [ ] **Step 3: 测试 Notification 事件（permission 子类型）**

Run:
```bash
echo '{"session_id":"test","transcript_path":"/tmp","cwd":"/Users/carlyu/my-project","hook_event_name":"Notification","notification_type":"permission"}' | ./target/release/ccnotify-rs Notification
```
Expected: macOS 弹出通知，标题为 "my-project"，副标题为 "Permission Required"

- [ ] **Step 4: 测试边界情况 — stdin 为空**

Run:
```bash
echo -n '' | ./target/release/ccnotify-rs Stop
```
Expected: 无通知弹出，进程静默退出（无 panic、无 stderr 错误输出）

- [ ] **Step 5: 测试边界情况 — 无效 JSON**

Run:
```bash
echo 'not-json' | ./target/release/ccnotify-rs Stop
```
Expected: 无通知弹出，进程静默退出

- [ ] **Step 6: 测试边界情况 — 未知事件**

Run:
```bash
echo '{}' | ./target/release/ccnotify-rs UnknownEvent
```
Expected: 无通知弹出，进程静默退出

- [ ] **Step 7: 根据测试结果决定是否需要修复**

如果发现以下问题，记录并创建修复任务：
- 点击 "Open" 后 Ghostty 未激活 → 检查 AppleScript fallback 是否生效
- 通知未弹出 → 检查 notify-rust 是否正确初始化
- 进程 30 秒后才退出 → 这是预期行为（recv_timeout 等待用户点击），Claude Hook 异步执行不影响主流程
- 通知弹出但无 "Open" 按钮 → 检查 `.action("default", "Open")` 是否生效

- [ ] **Step 8: Commit（如有修复）**

```bash
git add -A
git diff --cached --quiet || git commit -m "fix: address test findings"
```

---

### Task 7: 安装到 cargo bin 目录并配置 Hook

**Files:**
- 无文件变更

- [ ] **Step 1: 安装**

Run: `cargo install --path .`
Expected: 安装成功，输出 `Installed package ccnotify-rs v0.1.0`

- [ ] **Step 2: 验证安装路径**

Run: `ls -lh ~/.cargo/bin/ccnotify-rs`
Expected: 文件存在

- [ ] **Step 3: 验证 PATH 中可访问**

Run: `which ccnotify-rs`
Expected: 输出 `~/.cargo/bin/ccnotify-rs`

- [ ] **Step 4: 配置 Claude Code Hook（settings.json）**

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

注意：UserPromptSubmit 已从配置中移除（当前版本无操作，避免无意义进程 fork）。

---

### Task 8: 验证 cargo test 全通过

**Files:**
- 无文件变更

- [ ] **Step 1: 运行所有测试**

Run: `cargo test`
Expected: 所有单元测试 PASS（8 个测试：notifier 3 个 + event 5 个）

---

## Self-Review 检查清单

### 1. Spec 覆盖率

| Spec 要求 | 对应 Task |
|-----------|-----------|
| 三模块架构（main/event/notifier） | Task 2, 3, 4 |
| 零配置，纯硬编码 | Task 4（无配置文件读取） |
| 支持 Stop / Notification | Task 3（UserPromptSubmit 已移除） |
| Notification 子类型映射 | Task 3（map_notification_type） |
| 点击跳转 Ghostty | Task 2（jump_to_ghostty_terminal + fallback） |
| 静默失败，不阻塞 Claude | Task 4（所有错误用 `return` 或 `.ok()` 处理） |
| `test` 子命令 | Task 4 |
| Examples 测试 | Task 6 |
| AppleScript 双重 fallback | Task 2 |

**覆盖率：100%，无遗漏。**

### 2. Placeholder 扫描

- [x] 无 "TBD"/"TODO"/"implement later"
- [x] 无模糊描述（如 "add appropriate error handling"）
- [x] 每个代码步骤包含完整代码
- [x] 每个测试步骤包含完整测试代码
- [x] 无 "Similar to Task N" 引用

### 3. 类型一致性

| 类型/函数 | 定义位置 | 使用位置 | 状态 |
|-----------|----------|----------|------|
| `HookPayload` | Task 4 (main.rs) | Task 4 | ✅ 一致 |
| `notifier::notify(cwd: String, subtitle)` | Task 2 (notifier.rs) | Task 2, Task 3, Task 4 | ✅ 一致 |
| `event::handle(event, cwd: String, notification_type)` | Task 3 (event.rs) | Task 4 (main.rs) | ✅ 一致 |
| `extract_project_name` | Task 2 (private) | Task 2 | ✅ 一致 |
| `map_notification_type` | Task 3 (private) | Task 3 | ✅ 一致 |

**类型一致性：通过。**

---

## 执行选项

**Plan complete and saved to `docs/superpowers/plans/2026-04-26-ccnotify-rs.md`. Two execution options:**

**1. Subagent-Driven (recommended)** - I dispatch a fresh subagent per task, review between tasks, fast iteration

**2. Inline Execution** - Execute tasks in this session using executing-plans, batch execution with checkpoints

**Which approach?**
