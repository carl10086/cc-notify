# CCNotify UI 增强设计文档

> 日期：2026-04-27
> 目标：优化通知的 UI 显示，提升信息量和可识别性

## 1. 优化目标

1. **emoji 区分事件类型** — 一眼识别通知类型
2. **body 内容丰富化** — 显示更多上下文信息
3. **保留 🔒 前缀** — permission_mode == "ask" 时仍显示锁图标

## 2. 设计方案

### 2.1 emoji 标题前缀

| 事件 | emoji | 标题示例 |
|------|-------|----------|
| Stop | ⏹️ | ⏹️ my-project |
| Notification | 🔔 | 🔔 my-project |
| 未知事件 | ❓ | ❓ my-project |

### 2.2 body 内容

| 事件 | body 格式 |
|------|-----------|
| Stop | `{reason}\n{cwd}` |
| Notification | `{notification_type}\n{cwd}` |
| 未知事件 | `{cwd}` |

### 2.3 🔒 前缀保留

当 `permission_mode == "ask"` 时，在 subtitle 前加 🔒 前缀。

## 3. 通知效果示例

```
┌─────────────────────────────────────┐
│ ⏹️ my-project                       │
│ Stop: Task completed               │
│ Task completed                     │
│ /path/to/project                  │
└─────────────────────────────────────┘

┌─────────────────────────────────────┐
│ 🔔 my-project                       │
│ 🔒 Notification: Permission Required│
│ Permission Required                 │
│ /path/to/project                  │
└─────────────────────────────────────┘
```

## 4. 技术实现

- 修改文件：`src/event.rs`
- 修改函数：`build_notification_content`
- 测试：更新 `event::tests` 中的测试用例

## 5. 验收标准

1. Stop 事件标题显示 ⏹️ emoji
2. Notification 事件标题显示 🔔 emoji
3. 未知事件标题显示 ❓ emoji
4. Stop 事件 body 显示 reason + cwd
5. Notification 事件 body 显示 notification_type + cwd
6. permission_mode == "ask" 时 subtitle 显示 🔒 前缀
7. 所有测试通过
