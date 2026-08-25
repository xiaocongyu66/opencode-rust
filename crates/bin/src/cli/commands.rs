//! Command dispatch — `packages/cli/src/index.ts` + `commands/handlers/*`.
//!
//! Routes parsed CLI commands to their handlers, mirroring the TypeScript
//! `Runtime.handlers` + `Runtime.run` wiring.

use anyhow::Result;

use crate::cli::cli::{Cli, Commands, DebugAction, ServiceAction};
use crate::cli::services::Daemon;

pub async fn run(cli: Cli) -> Result<()> {
    crate::cli::i18n::init();
    init_rsopencode_dirs();
    let command = cli.command.unwrap_or(Commands::Default { resume: None });
    match command {
        Commands::Default { resume } => default_handler(resume).await,
        Commands::Api { request, data, header, param } => {
            api_handler(&request, data, &header, &param).await
        }
        Commands::Debug { action } => match action {
            DebugAction::Agents => debug_agents_handler().await,
        },
        Commands::Migrate => migrate_handler().await,
        Commands::Service { action } => service_handler(action).await,
        Commands::Serve { hostname, port, register } => {
            serve_handler(&hostname, port, register).await
        }
    }
}

// -- default ($ handler) ---------------------------------------------------

/// Default handler — launches the interactive TUI.
///
/// Mirrors `handlers/default.ts`: obtains the daemon transport and runs the TUI.
async fn default_handler(resume: Option<String>) -> Result<()> {
    crate::tui::app::run(resume).await
}

// -- serve -----------------------------------------------------------------

/// Serve handler — `handlers/serve.ts`.
///
/// Listens on the requested hostname/port (scanning from 4096 when no port is
/// given), optionally registers the server as the background daemon, then
/// blocks forever serving the API.
async fn serve_handler(hostname: &str, port: Option<u16>, register: bool) -> Result<()> {
    let daemon = Daemon::new()?;
    let password = daemon.password(None)?;

    let address = bind_address(hostname, port).await?;

    if register {
        let url = format!("http://{}", address);
        daemon.register(&url)?;
    }

    println!("server listening on {address}");

    let server = crate::server::Server::new();
    // The server is configured to authenticate with the generated password.
    Ok(server.serve_with_password(&address, &password).await?)
}

/// Resolves the bind address, scanning from 4096 upwards when no port is given.
///
/// Mirrors the `listen`/`bind`/`next` recursion in `handlers/serve.ts`.
async fn bind_address(hostname: &str, port: Option<u16>) -> Result<String> {
    match port {
        Some(p) => Ok(format!("{hostname}:{p}")),
        None => {
            let start = crate::cli::services::SERVE_PORT_START;
            for p in start..=u16::MAX {
                if is_port_available(hostname, p).await {
                    return Ok(format!("{hostname}:{p}"));
                }
            }
            anyhow::bail!("No available port in range {start}..{}", u16::MAX)
        }
    }
}

async fn is_port_available(hostname: &str, port: u16) -> bool {
    tokio::net::TcpListener::bind((hostname, port))
        .await
        .is_ok()
}

// -- service ---------------------------------------------------------------

/// Service handler — `handlers/service/*`.
async fn service_handler(action: ServiceAction) -> Result<()> {
    use ServiceAction::*;
    let daemon = Daemon::new()?;
    match action {
        Start => {
            let url = daemon.start()?;
            println!("{url}");
        }
        Restart => {
            let _ = daemon.stop();
            let url = daemon.start()?;
            println!("{url}");
        }
        Status => match daemon.status() {
            Some(url) => println!("running {url}"),
            None => println!("stopped"),
        },
        Stop => {
            daemon.stop()?;
        }
        Password { value } => {
            if value.is_some() {
                let _ = daemon.stop();
            }
            let pw = daemon.password(value.as_deref())?;
            println!("{pw}");
        }
    }
    Ok(())
}

// -- debug agents ----------------------------------------------------------

/// Debug agents handler — `handlers/debug/agents.ts`.
///
/// Lists agents via the daemon client, sorted by id.
async fn debug_agents_handler() -> Result<()> {
    let daemon = Daemon::new()?;
    let (url, auth) = daemon.transport()?;
    let client = reqwest::Client::new();
    let directory = std::env::current_dir()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_default();
    let resp = client
        .get(format!("{url}/api/agent"))
        .header("authorization", auth)
        .query(&[("location[directory]", directory.as_str())])
        .send()
        .await
        .map_err(|e| anyhow::anyhow!("Failed to list agents: {e}"))?;

    let body: serde_json::Value = resp.json().await.unwrap_or_default();
    let mut agents: Vec<serde_json::Value> = body
        .get("data")
        .and_then(|d| d.as_array())
        .cloned()
        .unwrap_or_default();
    agents.sort_by(|a, b| {
        a.get("id")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .cmp(b.get("id").and_then(|v| v.as_str()).unwrap_or(""))
    });
    println!("{}", serde_json::to_string_pretty(&agents)?);
    Ok(())
}

// -- api -------------------------------------------------------------------

/// API handler — `handlers/api.ts`.
///
/// Resolves the request (raw `method path` or OpenAPI operation ID) and
/// dispatches the HTTP call against the running server.
async fn api_handler(
    request: &[String],
    data: Option<String>,
    headers: &[String],
    params: &[(String, String)],
) -> Result<()> {
    let daemon = Daemon::new()?;
    let (base_url, auth) = daemon.transport()?;

    let resolved = resolve_request(&base_url, &auth, request, params).await?;

    let client = reqwest::Client::new();
    let mut req = client.request(
        reqwest::Method::from_bytes(resolved.method.as_bytes())
            .map_err(|e| anyhow::anyhow!("Invalid method: {e}"))?,
        format!("{}{}", base_url, resolved.path),
    );

    req = req.header("authorization", auth);

    let mut has_content_type = false;
    for header in headers {
        let idx = header
            .find(':')
            .ok_or_else(|| anyhow::anyhow!("Invalid header, expected name:value: {header}"))?;
        let name = header[..idx].trim();
        let value = header[idx + 1..].trim();
        if name.eq_ignore_ascii_case("content-type") {
            has_content_type = true;
        }
        req = req.header(name, value);
    }

    if let Some(body) = &data {
        if !has_content_type {
            req = req.header("content-type", "application/json");
        }
        req = req.body(body.clone());
    }

    let resp = req.send().await.map_err(|e| anyhow::anyhow!("Request failed: {e}"))?;
    let text = resp.text().await.unwrap_or_default();
    if !text.is_empty() {
        println!("{text}");
    }
    Ok(())
}

/// A resolved HTTP request (method + path).
struct ResolvedRequest {
    method: String,
    path: String,
}

/// Methods recognized as raw `method path` input.
const HTTP_METHODS: &[&str] = &["DELETE", "GET", "HEAD", "OPTIONS", "PATCH", "POST", "PUT"];

/// Parses raw `method path` input (two args).
fn raw_request(input: &[String]) -> Option<ResolvedRequest> {
    if input.len() != 2 {
        return None;
    }
    let method = input[0].to_uppercase();
    if !HTTP_METHODS.contains(&method.as_str()) {
        return None;
    }
    if !input[1].starts_with('/') {
        return None;
    }
    Some(ResolvedRequest { method, path: input[1].clone() })
}

/// Resolves either a raw `method path` pair or a single OpenAPI operation ID.
async fn resolve_request(
    base_url: &str,
    auth: &str,
    input: &[String],
    params: &[(String, String)],
) -> Result<ResolvedRequest> {
    if let Some(raw) = raw_request(input) {
        return Ok(raw);
    }
    if input.len() != 1 {
        anyhow::bail!("Expected an operation name or an HTTP method and path");
    }
    let operation_id = &input[0];
    let spec = fetch_openapi(base_url, auth).await?;
    resolve_operation(&spec, operation_id, params)
}

/// Fetches the OpenAPI document from the running server.
async fn fetch_openapi(base_url: &str, auth: &str) -> Result<serde_json::Value> {
    let client = reqwest::Client::new();
    let resp = client
        .get(format!("{base_url}/openapi.json"))
        .header("authorization", auth)
        .send()
        .await
        .map_err(|e| anyhow::anyhow!("Failed to load OpenAPI document: {e}"))?;
    if !resp.status().is_success() {
        anyhow::bail!("Failed to load OpenAPI document: HTTP {}", resp.status());
    }
    resp.json().await.map_err(Into::into)
}

/// Looks up an operation by ID in the OpenAPI spec and interpolates params.
fn resolve_operation(
    spec: &serde_json::Value,
    operation_id: &str,
    params: &[(String, String)],
) -> Result<ResolvedRequest> {
    let paths = spec
        .get("paths")
        .and_then(|p| p.as_object())
        .ok_or_else(|| anyhow::anyhow!("Operation not found: {operation_id}"))?;

    for (path, operations) in paths {
        let operations = match operations.as_object() {
            Some(o) => o,
            None => continue,
        };
        for (method, operation) in operations {
            if !HTTP_METHODS.iter().any(|m| m.eq_ignore_ascii_case(method)) {
                continue;
            }
            if operation.get("operationId").and_then(|v| v.as_str()) != Some(operation_id) {
                continue;
            }
            let interpolated = interpolate_path(path, params)?;
            return Ok(ResolvedRequest {
                method: method.to_uppercase(),
                path: interpolated,
            });
        }
    }
    anyhow::bail!("Operation not found: {operation_id}")
}

/// Interpolates `{param}` placeholders in a path and appends leftover params
/// as a query string.
fn interpolate_path(path: &str, params: &[(String, String)]) -> Result<String> {
    let mut used = std::collections::HashSet::new();
    let mut result = String::with_capacity(path.len());
    let mut chars = path.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '{' {
            let name: String = chars.by_ref().take_while(|&ch| ch != '}').collect();
            let value = params
                .iter()
                .find(|(k, _)| k == &name)
                .map(|(_, v)| v.clone())
                .ok_or_else(|| anyhow::anyhow!("Missing path parameter: {name}"))?;
            used.insert(name);
            result.push_str(&percent_encode(&value));
        } else {
            result.push(c);
        }
    }

    let query: Vec<(String, String)> = params
        .iter()
        .filter(|(k, _)| !used.contains(k))
        .cloned()
        .collect();
    if query.is_empty() {
        Ok(result)
    } else {
        let qs = query
            .iter()
            .map(|(k, v)| format!("{}={}", percent_encode(k), percent_encode(v)))
            .collect::<Vec<_>>()
            .join("&");
        Ok(format!("{result}?{qs}"))
    }
}

/// Percent-encodes a path or query segment.
fn percent_encode(input: &str) -> String {
    let mut result = String::with_capacity(input.len() * 3);
    for byte in input.bytes() {
        if byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.' | b'~') {
            result.push(byte as char);
        } else {
            result.push_str(&format!("%{:02X}", byte));
        }
    }
    result
}

// -- migrate ---------------------------------------------------------------

/// Migrate handler — `handlers/migrate.ts`.
async fn migrate_handler() -> Result<()> {
    tracing::info!("No migrations to run.");
    Ok(())
}

// -- .rsopencode init -------------------------------------------------------

/// Initializes `.rsopencode` configuration directories.
///
/// Creates the project-local `.rsopencode/` directory with a `sessions/`
/// subdirectory and a default `project.toml`, plus the global
/// `~/.rsopencode/` directory with a default `config.toml`. Existing files
/// are never overwritten.
fn init_rsopencode_dirs() {
    let dir = match crate::core::rsopencode::RsOpenCodeDir::from_cwd() {
        Ok(d) => d,
        Err(_) => return,
    };
    let _ = dir.ensure_project_dirs();
    let _ = dir.init_project_config();
    let _ = dir.ensure_global_dir();
    let _ = dir.init_global_config();
}
