mod config;
mod mcp;
mod tools;

use serde_json::{json, Value};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tracing::{error, info, warn};

use config::Config;
use mcp::protocol::{Capabilities, InitializeResult, Request, Response, ServerInfo, ToolCallResult, all_tools};

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_writer(std::io::stderr)
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_env("LINTER_LOG")
                .add_directive(tracing::Level::INFO.into()),
        )
        .init();

    let cfg = Config::from_env();
    info!(
        "kubevirt-project-linter starting. project root: {}",
        cfg.project_root.display()
    );

    run_server(cfg).await;
}

async fn run_server(cfg: Config) {
    let stdin = tokio::io::stdin();
    let mut stdout = tokio::io::stdout();
    let reader = BufReader::new(stdin);
    let mut lines = reader.lines();

    info!("MCP stdio server ready.");

    while let Ok(Some(line)) = lines.next_line().await {
        let line = line.trim().to_string();
        if line.is_empty() {
            continue;
        }

        let request: Request = match serde_json::from_str(&line) {
            Ok(r) => r,
            Err(e) => {
                let resp = Response::err(None, -32700, format!("Parse error: {}", e));
                send_response(&mut stdout, &resp).await;
                continue;
            }
        };

        if request.id.is_none() {
            continue;
        }

        let response = handle_request(request, &cfg).await;
        send_response(&mut stdout, &response).await;
    }

    info!("stdin closed, shutting down.");
}

async fn handle_request(req: Request, cfg: &Config) -> Response {
    let id = req.id.clone();

    match req.method.as_str() {
        "initialize" => {
            let result = InitializeResult {
                protocol_version: "2024-11-05".into(),
                capabilities: Capabilities { tools: json!({}) },
                server_info: ServerInfo {
                    name: "kubevirt-project-linter".into(),
                    version: env!("CARGO_PKG_VERSION").into(),
                },
            };
            Response::ok(id, serde_json::to_value(result).unwrap())
        }

        "tools/list" => Response::ok(id, json!({ "tools": all_tools() })),

        "tools/call" => {
            let params = req.params.unwrap_or(Value::Null);
            let tool_name = params.get("name").and_then(|v| v.as_str()).unwrap_or("");
            let tool_params =
                params.get("arguments").cloned().unwrap_or(Value::Object(Default::default()));

            let result = dispatch_tool(tool_name, &tool_params, cfg);
            Response::ok(id, serde_json::to_value(result).unwrap())
        }

        "ping" => Response::ok(id, json!({})),

        method => {
            warn!("Unknown method: {}", method);
            Response::err(id, -32601, format!("Method not found: {}", method))
        }
    }
}

fn dispatch_tool(name: &str, params: &Value, cfg: &Config) -> ToolCallResult {
    match name {
        "get_setup_rules" => tools::rules::get_setup_rules(cfg),
        "get_teardown_rules" => tools::rules::get_teardown_rules(cfg),
        "get_fixture_map" => tools::fixtures::get_fixture_map(cfg),
        "get_env_vars" => tools::fixtures::get_env_vars(cfg),
        "get_allure_suite_map" => tools::fixtures::get_allure_suite_map(cfg),
        "lint_spec_file" => tools::lint::lint_spec_file(params, cfg),
        "check_api_ui_parity" => tools::parity::check_api_ui_parity(cfg),
        "validate_std_coverage" => tools::parity::validate_std_coverage(cfg),
        unknown => ToolCallResult::error(format!(
            "Unknown tool: '{}'. Available: get_setup_rules, get_teardown_rules, get_fixture_map, get_env_vars, get_allure_suite_map, lint_spec_file, check_api_ui_parity, validate_std_coverage",
            unknown
        )),
    }
}

async fn send_response(stdout: &mut tokio::io::Stdout, response: &Response) {
    match serde_json::to_string(response) {
        Ok(json) => {
            let line = format!("{}\n", json);
            if let Err(e) = stdout.write_all(line.as_bytes()).await {
                error!("Failed to write response: {}", e);
            }
            if let Err(e) = stdout.flush().await {
                error!("Failed to flush stdout: {}", e);
            }
        }
        Err(e) => error!("Failed to serialize response: {}", e),
    }
}
