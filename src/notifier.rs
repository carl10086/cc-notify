use notify_rust::Notification;
use std::path::Path;
use std::process::Command;

/// 从 cwd 提取项目名作为通知标题
fn extract_project_name(cwd: &str) -> &str {
    Path::new(cwd)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or(cwd)
}

/// 发送通知
pub fn notify(cwd: &str, subtitle: &str) {
    let title = extract_project_name(cwd);

    let _ = Notification::new()
        .summary(title)
        .subtitle(subtitle)
        .show();
}

/// 跳转到 Ghostty 终端（双重 fallback）
/// 注意：macOS 通知中心不支持点击回调，此功能暂未启用
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
