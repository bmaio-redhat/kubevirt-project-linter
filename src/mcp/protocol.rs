use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

#[derive(Debug, Deserialize)]
pub struct Request {
    #[allow(dead_code)]
    pub jsonrpc: String,
    pub id: Option<Value>,
    pub method: String,
    pub params: Option<Value>,
}

#[derive(Debug, Serialize)]
pub struct Response {
    pub jsonrpc: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<RpcError>,
}

#[derive(Debug, Serialize)]
pub struct RpcError {
    pub code: i64,
    pub message: String,
}

impl Response {
    pub fn ok(id: Option<Value>, result: Value) -> Self {
        Self { jsonrpc: "2.0".into(), id, result: Some(result), error: None }
    }
    pub fn err(id: Option<Value>, code: i64, message: impl Into<String>) -> Self {
        Self {
            jsonrpc: "2.0".into(),
            id,
            result: None,
            error: Some(RpcError { code, message: message.into() }),
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ServerInfo {
    pub name: String,
    pub version: String,
}

#[derive(Debug, Serialize)]
pub struct Capabilities {
    pub tools: Value,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InitializeResult {
    pub protocol_version: String,
    pub capabilities: Capabilities,
    pub server_info: ServerInfo,
}

#[derive(Debug, Serialize)]
pub struct ContentItem {
    #[serde(rename = "type")]
    pub kind: String,
    pub text: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ToolCallResult {
    pub content: Vec<ContentItem>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_error: Option<bool>,
}

impl ToolCallResult {
    pub fn text(s: impl Into<String>) -> Self {
        Self {
            content: vec![ContentItem { kind: "text".into(), text: s.into() }],
            is_error: None,
        }
    }
    pub fn error(s: impl Into<String>) -> Self {
        Self {
            content: vec![ContentItem { kind: "text".into(), text: s.into() }],
            is_error: Some(true),
        }
    }
}

pub fn all_tools() -> Value {
    json!([
        {
            "name": "get_setup_rules",
            "description": "Parse project-dependencies/rule-engine/setup-rules.ts and return the authoritative list of setup rule IDs, phases (AUTH/CLUSTER/BROWSER/REPORTING), and descriptions. Use this instead of the README tables which are often out of sync.",
            "inputSchema": { "type": "object", "properties": {}, "required": [] }
        },
        {
            "name": "get_teardown_rules",
            "description": "Parse project-dependencies/teardown modules and return the authoritative list of teardown rule IDs and their cleanup targets. Use instead of the README tables.",
            "inputSchema": { "type": "object", "properties": {}, "required": [] }
        },
        {
            "name": "get_fixture_map",
            "description": "For every folder under playwright/tests/, return the fixture file it imports and the fixture function name. Authoritative replacement for the stale fixture table in README.",
            "inputSchema": { "type": "object", "properties": {}, "required": [] }
        },
        {
            "name": "get_env_vars",
            "description": "Parse the EnvVariables class and all usages in the project, returning an authoritative table of env var name, type, default value, and whether it is required. Replaces the .env.example docs.",
            "inputSchema": { "type": "object", "properties": {}, "required": [] }
        },
        {
            "name": "lint_spec_file",
            "description": "Check a spec file for convention violations: missing ID(CNV-XXXXX) annotations, raw 'page' usage instead of page object, hardcoded timeouts instead of TestTimeouts constants, missing cleanup.track calls after resource creation, and direct API calls without going through fixture clients.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "path": {
                        "type": "string",
                        "description": "Path to the spec file to lint (absolute or relative to KUBEVIRT_PROJECT_ROOT)"
                    }
                },
                "required": ["path"]
            }
        },
        {
            "name": "check_api_ui_parity",
            "description": "Compare tier1 UI spec files that make console-proxy write calls (POST/PUT/PATCH/DELETE) against the api/ test directory. Returns a list of UI tests that have console-proxy mutations but no corresponding API spec test.",
            "inputSchema": { "type": "object", "properties": {}, "required": [] }
        },
        {
            "name": "get_allure_suite_map",
            "description": "Parse allure-constants.ts and return the mapping of feature area → allure Epic/Feature/Story tags. Use when writing a new test to find the correct allure annotations.",
            "inputSchema": { "type": "object", "properties": {}, "required": [] }
        },
        {
            "name": "validate_std_coverage",
            "description": "Check that every ID(CNV-XXXXX) found in spec files has a corresponding entry in playwright/docs/ STD markdown files. Returns Jira IDs present in specs but missing from docs.",
            "inputSchema": { "type": "object", "properties": {}, "required": [] }
        }
    ])
}

// ── Unit tests ────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn response_ok_structure() {
        let r = Response::ok(Some(json!(1)), json!({"result": "data"}));
        assert_eq!(r.jsonrpc, "2.0");
        assert!(r.error.is_none());
    }

    #[test]
    fn response_err_no_result() {
        let r = Response::err(None, -32601, "not found");
        assert!(r.result.is_none());
        assert_eq!(r.error.as_ref().unwrap().code, -32601);
    }

    #[test]
    fn tool_result_text() {
        let t = ToolCallResult::text("ok");
        assert!(t.is_error.is_none());
        assert_eq!(t.content[0].text, "ok");
    }

    #[test]
    fn tool_result_error_flagged() {
        let t = ToolCallResult::error("bad");
        assert_eq!(t.is_error, Some(true));
    }

    #[test]
    fn all_eight_tools_present() {
        let tools = all_tools();
        let names: Vec<&str> = tools.as_array().unwrap()
            .iter()
            .filter_map(|t| t.get("name").and_then(|v| v.as_str()))
            .collect();
        for expected in &[
            "get_setup_rules", "get_teardown_rules", "get_fixture_map", "get_env_vars",
            "get_allure_suite_map", "lint_spec_file", "check_api_ui_parity", "validate_std_coverage",
        ] {
            assert!(names.contains(expected), "missing tool: {}", expected);
        }
    }

    #[test]
    fn all_tools_have_valid_schemas() {
        for tool in all_tools().as_array().unwrap() {
            let name = tool.get("name").and_then(|v| v.as_str()).unwrap_or("?");
            assert_eq!(
                tool.get("inputSchema").and_then(|s| s.get("type")).and_then(|v| v.as_str()),
                Some("object"),
                "tool '{}' schema type should be object", name
            );
        }
    }
}
