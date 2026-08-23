use std::ffi::OsStr;
use std::io;
use std::os::windows::process::CommandExt;
use std::path::Path;
use std::process::{Command, Output};

use serde_json::Value;

use crate::constants::{CREATE_NO_WINDOW, FALLBACK_GLAZEWM_PATH};
use crate::model::WindowInfo;

pub(crate) fn query_focused_window() -> Result<WindowInfo, String> {
    let output = run_glazewm_query().map_err(|error| {
        format!(
            "无法运行 `glazewm query focused`：{error}\n请确认 GlazeWM 正在运行并且 CLI 位于 PATH 中。"
        )
    })?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_owned();
        return Err(if stderr.is_empty() {
            format!("GlazeWM 查询失败，退出码：{}", output.status)
        } else {
            format!("GlazeWM 查询失败：{stderr}")
        });
    }

    parse_query_output(&output.stdout)
}

fn parse_query_output(output: &[u8]) -> Result<WindowInfo, String> {
    let response: Value = serde_json::from_slice(output)
        .map_err(|error| format!("无法解析 GlazeWM 返回的 JSON：{error}"))?;

    if response.get("success").and_then(Value::as_bool) == Some(false) {
        let message = response
            .get("error")
            .and_then(Value::as_str)
            .unwrap_or("GlazeWM 返回了未知错误。");
        return Err(message.to_owned());
    }

    let focused = response.pointer("/data/focused");
    let field = |name| {
        focused
            .and_then(|value| value.get(name))
            .and_then(Value::as_str)
            .unwrap_or("—")
            .to_owned()
    };

    let formatted_json = serde_json::to_string_pretty(&response)
        .map_err(|error| format!("无法格式化 GlazeWM 返回的 JSON：{error}"))?;

    Ok(WindowInfo {
        title: field("title"),
        class_name: field("className"),
        process_name: field("processName"),
        formatted_json,
    })
}

fn run_glazewm_query() -> io::Result<Output> {
    match run_query_command("glazewm") {
        Ok(output) => Ok(output),
        Err(error) if error.kind() == io::ErrorKind::NotFound => {
            if Path::new(FALLBACK_GLAZEWM_PATH).is_file() {
                run_query_command(FALLBACK_GLAZEWM_PATH)
            } else {
                Err(error)
            }
        }
        Err(error) => Err(error),
    }
}

fn run_query_command(program: impl AsRef<OsStr>) -> io::Result<Output> {
    Command::new(program)
        .args(["query", "focused"])
        .creation_flags(CREATE_NO_WINDOW)
        .output()
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE_RESPONSE: &[u8] = br#"{
        "clientMessage": "query focused",
        "data": {
            "focused": {
                "type": "window",
                "handle": 589994,
                "title": "main.rs",
                "className": "Chrome_WidgetWin_1",
                "processName": "Code"
            }
        },
        "error": null,
        "success": true
    }"#;

    #[test]
    fn extracts_summary_fields() {
        let result = parse_query_output(SAMPLE_RESPONSE).unwrap();
        assert_eq!(result.process_name, "Code");
        assert_eq!(result.class_name, "Chrome_WidgetWin_1");
        assert_eq!(result.title, "main.rs");
    }

    #[test]
    fn preserves_and_formats_the_complete_response() {
        let result = parse_query_output(SAMPLE_RESPONSE).unwrap();
        assert!(
            result
                .formatted_json
                .contains("\"clientMessage\": \"query focused\"")
        );
        assert!(result.formatted_json.contains("\n  \"data\""));
        assert!(result.formatted_json.contains("\"handle\": 589994"));
    }
}
