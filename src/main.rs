use std::io::{self, Read};

mod event;
mod notifier;

#[derive(Debug, serde::Deserialize)]
struct HookPayload {
    session_id: String,
    transcript_path: String,
    cwd: String,
    permission_mode: String,           // 新增
    hook_event_name: String,
    reason: Option<String>,           // 新增：Stop 事件特有
    #[serde(default)]
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
        notifier::notify(&cwd, "Test Notification");
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

    event::handle(event_name, &payload.cwd, payload.notification_type.as_deref());
}
