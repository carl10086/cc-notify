use crate::notifier;

/// 处理事件，根据 event_name 发送对应通知
pub fn handle(event_name: &str, cwd: &str, notification_type: Option<&str>) {
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
