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
            (format!("{}Stop: {}", prefix, reason_text), Some("Default".to_string()))
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
            (format!("Notification: {}{}", prefix, detail), Some(sound.to_string()))
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
        assert_eq!(content.sound, Some("Default".to_string()));
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
        assert_eq!(content.sound, Some("Breeze".to_string()));
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