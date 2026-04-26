# UI 增强实现计划

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 为通知添加 emoji 前缀和丰富的 body 内容

**Architecture:** 修改 `src/event.rs` 的 `build_notification_content` 函数，更新 title 和 body 格式

**Tech Stack:** Rust

---

## 文件变更

- Modify: `src/event.rs` — 更新 `build_notification_content` 和测试用例

---

### Task 1: 更新 build_notification_content 函数

**Files:**
- Modify: `src/event.rs:15-53`

- [ ] **Step 1: 修改 Stop 事件分支**

将 Stop 事件返回值从 `(subtitle, sound)` 改为 `(title_prefix, subtitle, sound, body)`：

```rust
"Stop" => {
    let reason_text = reason.unwrap_or("Unknown reason");
    let lock_prefix = if permission_mode == "ask" { "🔒 " } else { "" };
    let subtitle = format!("{}Stop: {}", lock_prefix, reason_text);
    let body = format!("{}\n{}", reason_text, cwd);
    ("⏹️", subtitle, Some("Default".to_string()), body)
}
```

- [ ] **Step 2: 修改 Notification 事件分支**

```rust
"Notification" => {
    let sub_type = notification_type.unwrap_or("Notification");
    let (detail, sound) = match sub_type {
        "permission" => ("Permission Required", "Breeze"),
        "input" => ("Waiting for input", "Default"),
        "action" => ("Action Required", "Default"),
        _ => ("Notification", "Default"),
    };
    let subtitle = if permission_mode == "ask" {
        format!("🔒 Notification: {}", detail)
    } else {
        format!("Notification: {}", detail)
    };
    let body = format!("{}\n{}", detail, cwd);
    ("🔔", subtitle, Some(sound.to_string()), body)
}
```

- [ ] **Step 3: 修改未知事件分支**

```rust
_ => {
    let lock_prefix = if permission_mode == "ask" { "🔒 " } else { "" };
    let subtitle = format!("{}{}", lock_prefix, event_name);
    let body = cwd.to_string();
    ("❓", subtitle, None, body)
}
```

- [ ] **Step 4: 修改返回语句**

将 `NotificationContent` 构建改为：

```rust
NotificationContent {
    title: format!("{} {}", title_prefix, title),
    subtitle,
    body,
    sound,
}
```

- [ ] **Step 5: 运行测试验证**

Run: `cargo test 2>&1`
Expected: 10 个测试全部 PASS

- [ ] **Step 6: Commit**

```bash
git add src/event.rs
git commit -m "feat(ui): add emoji prefixes and enriched body content"
```

---

### Task 2: 更新测试用例

**Files:**
- Modify: `src/event.rs:63-122`

- [ ] **Step 1: 更新 test_stop_with_reason**

```rust
#[test]
fn test_stop_with_reason() {
    let content = build_notification_content("Stop", "/Users/carlyu/my-project", Some("Task completed"), "allow", None);
    assert_eq!(content.title, "⏹️ my-project");
    assert_eq!(content.subtitle, "Stop: Task completed");
    assert_eq!(content.body, "Task completed\n/Users/carlyu/my-project");
    assert_eq!(content.sound, Some("Default".to_string()));
}
```

- [ ] **Step 2: 更新 test_stop_without_reason**

```rust
#[test]
fn test_stop_without_reason() {
    let content = build_notification_content("Stop", "/Users/carlyu/my-project", None, "allow", None);
    assert_eq!(content.subtitle, "Stop: Unknown reason");
    assert_eq!(content.body, "Unknown reason\n/Users/carlyu/my-project");
}
```

- [ ] **Step 3: 更新 test_stop_with_permission_ask**

```rust
#[test]
fn test_stop_with_permission_ask() {
    let content = build_notification_content("Stop", "/Users/carlyu/my-project", Some("Task completed"), "ask", None);
    assert_eq!(content.title, "⏹️ my-project");
    assert_eq!(content.subtitle, "🔒 Stop: Task completed");
}
```

- [ ] **Step 4: 更新 test_notification_permission**

```rust
#[test]
fn test_notification_permission() {
    let content = build_notification_content("Notification", "/Users/carlyu/my-project", None, "allow", Some("permission"));
    assert_eq!(content.title, "🔔 my-project");
    assert_eq!(content.subtitle, "Notification: Permission Required");
    assert_eq!(content.body, "Permission Required\n/Users/carlyu/my-project");
    assert_eq!(content.sound, Some("Breeze".to_string()));
}
```

- [ ] **Step 5: 更新 test_notification_input**

```rust
#[test]
fn test_notification_input() {
    let content = build_notification_content("Notification", "/Users/carlyu/my-project", None, "allow", Some("input"));
    assert_eq!(content.title, "🔔 my-project");
    assert_eq!(content.subtitle, "Notification: Waiting for input");
    assert_eq!(content.body, "Waiting for input\n/Users/carlyu/my-project");
}
```

- [ ] **Step 6: 更新 test_unknown_event**

```rust
#[test]
fn test_unknown_event() {
    let content = build_notification_content("UnknownEvent", "/Users/carlyu/my-project", None, "allow", None);
    assert_eq!(content.title, "❓ my-project");
    assert_eq!(content.subtitle, "UnknownEvent");
    assert_eq!(content.body, "/Users/carlyu/my-project");
}
```

- [ ] **Step 7: 运行测试验证**

Run: `cargo test 2>&1`
Expected: 10 个测试全部 PASS

- [ ] **Step 8: Commit**

```bash
git add src/event.rs
git commit -m "test: update tests for UI enhancement"
```

---

## Self-Review 检查清单

### 1. Spec 覆盖率

| Spec 要求 | 对应 Task |
|-----------|-----------|
| Stop 标题显示 ⏹️ emoji | Task 1 |
| Notification 标题显示 🔔 emoji | Task 1 |
| 未知事件显示 ❓ emoji | Task 1 |
| body 显示 reason + cwd | Task 1 |
| body 显示 notification_type + cwd | Task 1 |
| 🔒 前缀保留 | Task 1 |

**覆盖率：100%**

### 2. Placeholder 扫描

- [x] 无 "TBD"/"TODO"/"implement later"
- [x] 每个步骤包含完整代码
- [x] 无模糊描述

---

## 执行选项

**Plan complete and saved to `docs/superpowers/plans/2026-04-27-ccnotify-rs-ui-enhancement.md`. Two execution options:**

**1. Subagent-Driven (recommended)** - I dispatch a fresh subagent per task, review between tasks, fast iteration

**2. Inline Execution** - Execute tasks in this session using executing-plans, batch execution with checkpoints

**Which approach?**
