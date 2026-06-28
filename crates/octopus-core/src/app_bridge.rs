use super::{
    bootstrap_workspace, default_tentacles_root, provider_profile, repo_root, valid_env_prefix,
    BootstrapReport, DEFAULT_PROVIDER_ENV_PATH,
};
use crate::release_gate::{preflight_check, PreflightCheck};
use std::env;
use std::fs;
use std::io::{BufRead, BufReader, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct StartOptions {
    pub(crate) addr: String,
    pub(crate) open: bool,
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub(crate) struct LocalAppRunReport {
    pub(crate) version: String,
    pub(crate) state_path: String,
    pub(crate) record_path: String,
    pub(crate) current_head: Option<String>,
    pub(crate) app_url: String,
    pub(crate) ready: bool,
    pub(crate) installed_tentacles: Vec<String>,
    pub(crate) pages: Vec<LocalAppPageReport>,
    pub(crate) web_demo: PreflightCheck,
    pub(crate) api_policy: PreflightCheck,
    pub(crate) next: Vec<String>,
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub(crate) struct LocalAppPageReport {
    pub(crate) path: String,
    pub(crate) ok: bool,
    pub(crate) bytes: usize,
    pub(crate) marker: String,
}

#[derive(serde::Deserialize)]
pub(crate) struct RunRequest {
    pub(crate) args: Vec<String>,
}

#[derive(serde::Serialize)]
pub(crate) struct RunResponse {
    pub(crate) ok: bool,
    pub(crate) code: Option<i32>,
    pub(crate) stdout: String,
    pub(crate) stderr: String,
    pub(crate) policy: Option<String>,
    pub(crate) suggested_args: Vec<Vec<String>>,
}

pub(crate) fn run_start(rest: &[String], state_path: PathBuf) -> Result<(), String> {
    let options = parse_start_options(rest)?;
    run_bridge(&options.addr, state_path, options.open)
}

pub(crate) fn start_check_requested(rest: &[String]) -> bool {
    rest.iter()
        .skip(1)
        .any(|value| value == "--check" || value == "check")
}

pub(crate) fn parse_start_check_addr(rest: &[String]) -> Result<String, String> {
    let mut addr = None;
    for value in rest.iter().skip(1) {
        match value.as_str() {
            "--check" | "check" => {}
            "--open" => return Err("start --check cannot be combined with --open".to_string()),
            option if option.starts_with("--") => {
                return Err(format!("unknown start --check option: {option}"));
            }
            value => {
                if addr.replace(value.to_string()).is_some() {
                    return Err("start --check accepts at most one address".to_string());
                }
            }
        }
    }
    Ok(addr.unwrap_or_else(|| "127.0.0.1:8765".to_string()))
}

pub(crate) fn write_local_app_run_report(
    state_path: PathBuf,
    addr: String,
    current_head: Option<String>,
) -> Result<LocalAppRunReport, String> {
    let report = local_app_run_report(state_path, addr, current_head)?;
    let path = PathBuf::from(&report.record_path);
    if let Some(parent) = path.parent().filter(|path| !path.as_os_str().is_empty()) {
        fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    }
    fs::write(
        &path,
        serde_json::to_vec_pretty(&report).map_err(|error| error.to_string())?,
    )
    .map_err(|error| error.to_string())?;
    Ok(report)
}

pub(crate) fn local_app_run_status_check(
    state_path: &Path,
    current_head: Option<&str>,
) -> PreflightCheck {
    let path = local_app_run_record_path(state_path);
    let next = format!(
        "octopus --state {} start --check",
        shell_arg(&state_path.to_string_lossy())
    );
    let Ok(content) = fs::read_to_string(&path) else {
        return preflight_check(
            "local_app_run",
            false,
            true,
            format!("record missing: {}", path.to_string_lossy()),
            next,
        );
    };
    let report = match serde_json::from_str::<LocalAppRunReport>(&content) {
        Ok(report) => report,
        Err(error) => {
            return preflight_check(
                "local_app_run",
                false,
                true,
                format!("record parse failed: {error}"),
                next,
            );
        }
    };
    let head_ready = match current_head {
        Some(head) => report.current_head.as_deref() == Some(head),
        None => report.current_head.is_none(),
    };
    let version_ready = report.version == env!("CARGO_PKG_VERSION");
    let pages_ready = report.pages.iter().all(|page| page.ok);
    let web_demo_ready = report.web_demo.status == "pass";
    preflight_check(
        "local_app_run",
        report.ready && head_ready && version_ready && pages_ready && web_demo_ready,
        true,
        format!(
            "ready={}, head={}, version={}, pages={}/{}, web_demo={}, policy={}",
            report.ready,
            report.current_head.as_deref().unwrap_or("unknown"),
            report.version,
            report.pages.iter().filter(|page| page.ok).count(),
            report.pages.len(),
            report.web_demo.status,
            report.api_policy.status
        ),
        next,
    )
}

fn local_app_run_report(
    state_path: PathBuf,
    addr: String,
    current_head: Option<String>,
) -> Result<LocalAppRunReport, String> {
    let startup = prepare_state(state_path.clone())?;
    let pages = local_app_pages();
    let web_demo = web_demo_preflight_check();
    let api_policy = goal_surface_preflight_check(&state_path);
    let pages_ready = pages.iter().all(|page| page.ok);
    let ready = Path::new(&startup.state_path).exists()
        && !startup.installed_tentacles.is_empty()
        && pages_ready
        && web_demo.status == "pass"
        && api_policy.status == "pass";
    let mut next = vec![
        format!(
            "octopus --state {} preflight",
            shell_arg(&startup.state_path)
        ),
        format!("octopus start {addr}"),
    ];
    if !ready {
        next.push("octopus bootstrap".to_string());
    }
    next.sort();
    next.dedup();
    Ok(LocalAppRunReport {
        version: env!("CARGO_PKG_VERSION").to_string(),
        state_path: startup.state_path,
        record_path: local_app_run_record_path(&state_path)
            .to_string_lossy()
            .to_string(),
        current_head,
        app_url: format!("http://{addr}/app.html"),
        ready,
        installed_tentacles: startup.installed_tentacles,
        pages,
        web_demo,
        api_policy,
        next,
    })
}

fn web_demo_preflight_check() -> PreflightCheck {
    let (_, body) = match static_page("/app.html") {
        Ok(page) => page,
        Err(error) => {
            return preflight_check(
                "web_try_app",
                false,
                true,
                format!("app page unavailable: {error}"),
                "restore docs/app.html",
            );
        }
    };
    let content = String::from_utf8_lossy(&body);
    let markers = [
        r#"id="apiKey""#,
        "Hello World",
        "Draw Octopus",
        "clean brain only returns a Need",
        "browserTentaclePlan",
        "chatCompletionsEndpoint",
        "renderOctopusAnimation",
    ];
    let missing = markers
        .iter()
        .filter(|marker| !content.contains(**marker))
        .copied()
        .collect::<Vec<_>>();
    preflight_check(
        "web_try_app",
        missing.is_empty(),
        true,
        if missing.is_empty() {
            "api-key Need demo and browser-tentacle Feed demo present".to_string()
        } else {
            format!("missing {}", missing.join(", "))
        },
        "restore browser Try App demo in docs/app.html",
    )
}

fn local_app_pages() -> Vec<LocalAppPageReport> {
    [
        ("/app.html", "Octopus App"),
        ("/pet.html", "pixel-pet"),
        ("/index.html", "Octopus"),
        ("/tutorial.html", "Octopus Tutorial"),
        ("/use.html", "Use Octopus"),
    ]
    .into_iter()
    .map(|(path, marker)| {
        let (ok, bytes) = static_page(path)
            .map(|(_, body)| {
                let ok = String::from_utf8_lossy(&body).contains(marker);
                (ok, body.len())
            })
            .unwrap_or((false, 0));
        LocalAppPageReport {
            path: path.to_string(),
            ok,
            bytes,
            marker: marker.to_string(),
        }
    })
    .collect()
}

fn local_app_run_record_path(state_path: &Path) -> PathBuf {
    state_path
        .parent()
        .filter(|path| !path.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."))
        .join("local-app-run.json")
}

pub(crate) fn goal_surface_preflight_check(state_path: &Path) -> PreflightCheck {
    let state = state_path.to_string_lossy().to_string();
    let allowed_cases = [
        vec![
            "--state".to_string(),
            state.clone(),
            "--json".to_string(),
            "chat".to_string(),
            "refine the goal".to_string(),
        ],
        vec![
            "--state".to_string(),
            state.clone(),
            "--json".to_string(),
            "goal".to_string(),
            "refine".to_string(),
            "prefer clean Needs".to_string(),
        ],
        vec![
            "--state".to_string(),
            state.clone(),
            "--json".to_string(),
            "brain".to_string(),
            "--goal".to_string(),
            "--save".to_string(),
            "tighten the objective".to_string(),
        ],
        vec![
            "--state".to_string(),
            state.clone(),
            "--json".to_string(),
            "first-run".to_string(),
            "make this repo easier".to_string(),
        ],
    ];
    let denied_cases = [
        vec![
            "--state".to_string(),
            state.clone(),
            "--json".to_string(),
            "need".to_string(),
            "observe".to_string(),
            ".".to_string(),
        ],
        vec![
            "--state".to_string(),
            state.clone(),
            "--json".to_string(),
            "repair".to_string(),
            ".".to_string(),
        ],
        vec![
            "--state".to_string(),
            state.clone(),
            "--json".to_string(),
            "provider".to_string(),
            "save".to_string(),
            "openai".to_string(),
            "OCTOPUS_LLM".to_string(),
        ],
        vec![
            "--state".to_string(),
            state,
            "--json".to_string(),
            "preflight".to_string(),
            "record".to_string(),
            "append".to_string(),
        ],
    ];
    let allowed_count = allowed_cases
        .iter()
        .filter(|args| command_allowed(args))
        .count();
    let denied_count = denied_cases
        .iter()
        .filter(|args| !command_allowed(args))
        .count();
    let denied = denied_response(&denied_cases[0]);
    let policy_ok = !denied.ok
        && denied.policy.as_deref() == Some("user_writes_brain_goal_only")
        && denied
            .suggested_args
            .iter()
            .any(|args| args.iter().any(|arg| arg == "--goal"));
    preflight_check(
        "bridge_goal_surface",
        allowed_count == allowed_cases.len() && denied_count == denied_cases.len() && policy_ok,
        true,
        format!(
            "allowed_goal_writes={allowed_count}/{}, denied_internal_writes={denied_count}/{}, policy={}",
            allowed_cases.len(),
            denied_cases.len(),
            denied.policy.as_deref().unwrap_or("missing")
        ),
        "octopus chat \"refine the goal\"",
    )
}

pub(crate) fn parse_start_options(rest: &[String]) -> Result<StartOptions, String> {
    let mut addr = None;
    let mut open = false;
    for value in rest.iter().skip(1) {
        match value.as_str() {
            "--open" => open = true,
            option if option.starts_with("--") => {
                return Err(format!("unknown start option: {option}"));
            }
            value => {
                if addr.replace(value.to_string()).is_some() {
                    return Err("start accepts at most one address".to_string());
                }
            }
        }
    }
    Ok(StartOptions {
        addr: addr.unwrap_or_else(|| "127.0.0.1:8765".to_string()),
        open,
    })
}

fn run_bridge(addr: &str, state_path: PathBuf, open_app: bool) -> Result<(), String> {
    let listener =
        TcpListener::bind(addr).map_err(|error| format!("start bind failed: {error}"))?;
    let startup = prepare_state(state_path)?;
    print_startup(&startup, addr);
    if open_app {
        let url = format!("http://{addr}/app.html");
        if let Err(error) = open_app_url(&url) {
            eprintln!("open warning: {error}");
        }
    }
    for stream in listener.incoming() {
        match stream {
            Ok(mut stream) => {
                if let Err(error) = handle_connection(&mut stream) {
                    let body = serde_json::json!({ "error": error }).to_string();
                    let _ =
                        write_http_response(&mut stream, 500, "application/json", body.as_bytes());
                }
            }
            Err(error) => eprintln!("local app connection failed: {error}"),
        }
    }
    Ok(())
}

fn open_app_url(url: &str) -> Result<(), String> {
    let (command, args) = app_open_command(url);
    let status = Command::new(command)
        .args(args)
        .status()
        .map_err(|error| format!("{command} failed: {error}"))?;
    if status.success() {
        Ok(())
    } else {
        Err(format!("{command} exited with {status}"))
    }
}

pub(crate) fn app_open_command(url: &str) -> (&'static str, Vec<String>) {
    if cfg!(target_os = "macos") {
        ("open", vec![url.to_string()])
    } else if cfg!(target_os = "windows") {
        (
            "cmd",
            vec![
                "/C".to_string(),
                "start".to_string(),
                "".to_string(),
                url.to_string(),
            ],
        )
    } else {
        ("xdg-open", vec![url.to_string()])
    }
}

pub(crate) fn prepare_state(state_path: PathBuf) -> Result<BootstrapReport, String> {
    bootstrap_workspace(state_path, default_tentacles_root())
}

fn print_startup(report: &BootstrapReport, addr: &str) {
    println!("Octopus start");
    println!("state: {}", report.state_path);
    println!("seeds: {}", join_or_none(&report.seed_tentacles));
    println!("installed: {}", join_or_none(&report.installed_tentacles));
    println!("app: http://{addr}/app.html");
    println!("api: http://{addr}");
}

fn join_or_none(values: &[String]) -> String {
    if values.is_empty() {
        "none".to_string()
    } else {
        values.join(", ")
    }
}

fn shell_arg(value: &str) -> String {
    format!("'{}'", value.replace('\'', "'\\''"))
}

fn handle_connection(stream: &mut TcpStream) -> Result<(), String> {
    let request = read_http_request(stream)?;
    match (request.method.as_str(), request.path.as_str()) {
        ("OPTIONS", "/api/run" | "/api/stream") => {
            write_http_response(stream, 204, "text/plain", b"")
        }
        ("POST", "/api/run") => {
            let request = serde_json::from_slice::<RunRequest>(&request.body)
                .map_err(|error| format!("invalid bridge JSON: {error}"))?;
            let response = run_command(&request.args)?;
            let body = serde_json::to_vec_pretty(&response).map_err(|error| error.to_string())?;
            write_http_response(stream, 200, "application/json", &body)
        }
        ("POST", "/api/stream") => {
            let request = serde_json::from_slice::<RunRequest>(&request.body)
                .map_err(|error| format!("invalid bridge JSON: {error}"))?;
            stream_command(stream, &request.args)
        }
        ("GET", path) => {
            if let Ok((content_type, body)) = static_page(path) {
                write_http_response(stream, 200, content_type, &body)
            } else {
                write_http_response(stream, 404, "text/plain", b"not found")
            }
        }
        _ => write_http_response(stream, 404, "text/plain", b"not found"),
    }
}

struct HttpRequest {
    method: String,
    path: String,
    body: Vec<u8>,
}

fn read_http_request(stream: &mut TcpStream) -> Result<HttpRequest, String> {
    stream
        .set_read_timeout(Some(Duration::from_secs(5)))
        .map_err(|error| error.to_string())?;
    let mut data = Vec::new();
    let mut buffer = [0_u8; 4096];
    let header_end = loop {
        let bytes = stream
            .read(&mut buffer)
            .map_err(|error| error.to_string())?;
        if bytes == 0 {
            return Err("empty bridge request".to_string());
        }
        data.extend_from_slice(&buffer[..bytes]);
        if let Some(index) = find_header_end(&data) {
            break index;
        }
        if data.len() > 64 * 1024 {
            return Err("bridge request headers too large".to_string());
        }
    };
    let header = String::from_utf8_lossy(&data[..header_end]).to_string();
    let content_length = http_content_length(&header)?;
    while data.len() < header_end + 4 + content_length {
        let bytes = stream
            .read(&mut buffer)
            .map_err(|error| error.to_string())?;
        if bytes == 0 {
            break;
        }
        data.extend_from_slice(&buffer[..bytes]);
    }
    let request_line = header
        .lines()
        .next()
        .ok_or_else(|| "missing request line".to_string())?;
    let mut parts = request_line.split_whitespace();
    let method = parts
        .next()
        .ok_or_else(|| "missing method".to_string())?
        .to_string();
    let path = parts
        .next()
        .ok_or_else(|| "missing path".to_string())?
        .split('?')
        .next()
        .unwrap_or("/")
        .to_string();
    let body_start = header_end + 4;
    let body_end = (body_start + content_length).min(data.len());
    Ok(HttpRequest {
        method,
        path,
        body: data[body_start..body_end].to_vec(),
    })
}

fn find_header_end(data: &[u8]) -> Option<usize> {
    data.windows(4).position(|window| window == b"\r\n\r\n")
}

pub(crate) fn http_content_length(header: &str) -> Result<usize, String> {
    header
        .lines()
        .find_map(|line| {
            let (name, value) = line.split_once(':')?;
            name.eq_ignore_ascii_case("content-length")
                .then(|| value.trim().parse::<usize>())
        })
        .transpose()
        .map_err(|_| "invalid content-length".to_string())
        .map(|value| value.unwrap_or(0))
}

fn write_http_response(
    stream: &mut TcpStream,
    status: u16,
    content_type: &str,
    body: &[u8],
) -> Result<(), String> {
    let reason = match status {
        200 => "OK",
        204 => "No Content",
        404 => "Not Found",
        _ => "Error",
    };
    let header = format!(
        "HTTP/1.1 {status} {reason}\r\ncontent-type: {content_type}; charset=utf-8\r\ncontent-length: {}\r\naccess-control-allow-origin: *\r\naccess-control-allow-headers: content-type\r\naccess-control-allow-methods: GET, POST, OPTIONS\r\nconnection: close\r\n\r\n",
        body.len()
    );
    stream
        .write_all(header.as_bytes())
        .and_then(|_| stream.write_all(body))
        .map_err(|error| error.to_string())
}

fn write_http_stream_header(stream: &mut TcpStream) -> Result<(), String> {
    stream
        .write_all(
            b"HTTP/1.1 200 OK\r\ncontent-type: text/event-stream; charset=utf-8\r\ncache-control: no-cache\r\naccess-control-allow-origin: *\r\naccess-control-allow-headers: content-type\r\naccess-control-allow-methods: GET, POST, OPTIONS\r\nconnection: close\r\n\r\n",
        )
        .map_err(|error| error.to_string())
}

fn write_sse_event(
    stream: &mut TcpStream,
    event: &str,
    value: &serde_json::Value,
) -> Result<(), String> {
    let data = serde_json::to_string(value).map_err(|error| error.to_string())?;
    stream
        .write_all(format!("event: {event}\ndata: {data}\n\n").as_bytes())
        .and_then(|_| stream.flush())
        .map_err(|error| error.to_string())
}

pub(crate) fn static_page(path: &str) -> Result<(&'static str, Vec<u8>), String> {
    let Some((file, embedded)) = static_asset(path) else {
        return Err("bridge page not found".to_string());
    };
    let path = repo_root().join("docs").join(file);
    let body = fs::read(path).unwrap_or_else(|_| embedded.to_vec());
    Ok(("text/html", body))
}

pub(crate) fn static_asset(path: &str) -> Option<(&'static str, &'static [u8])> {
    match path {
        "/" | "/app.html" => Some(("app.html", &include_bytes!("../../../docs/app.html")[..])),
        "/index.html" => Some((
            "index.html",
            &include_bytes!("../../../docs/index.html")[..],
        )),
        "/pet.html" => Some(("pet.html", &include_bytes!("../../../docs/pet.html")[..])),
        "/quickstart.html" => Some((
            "quickstart.html",
            &include_bytes!("../../../docs/quickstart.html")[..],
        )),
        "/tutorial.html" => Some((
            "tutorial.html",
            &include_bytes!("../../../docs/tutorial.html")[..],
        )),
        "/use.html" => Some(("use.html", &include_bytes!("../../../docs/use.html")[..])),
        "/about.html" => Some((
            "about.html",
            &include_bytes!("../../../docs/about.html")[..],
        )),
        "/references.html" => Some((
            "references.html",
            &include_bytes!("../../../docs/references.html")[..],
        )),
        "/self-iteration.html" => Some((
            "self-iteration.html",
            &include_bytes!("../../../docs/self-iteration.html")[..],
        )),
        _ => None,
    }
}

pub(crate) fn run_command(args: &[String]) -> Result<RunResponse, String> {
    if !command_allowed(args) {
        return Ok(denied_response(args));
    }
    let mut command = Command::new(env::current_exe().map_err(|error| error.to_string())?);
    command.args(args);
    apply_env_overlay(&mut command);
    let output = command
        .output()
        .map_err(|error| format!("local app command failed to start: {error}"))?;
    Ok(RunResponse {
        ok: output.status.success(),
        code: output.status.code(),
        stdout: String::from_utf8_lossy(&output.stdout).to_string(),
        stderr: String::from_utf8_lossy(&output.stderr).to_string(),
        policy: None,
        suggested_args: Vec::new(),
    })
}

fn stream_command(stream: &mut TcpStream, args: &[String]) -> Result<(), String> {
    if !command_allowed(args) {
        write_http_stream_header(stream)?;
        let denied = denied_response(args);
        write_sse_event(
            stream,
            "stderr",
            &serde_json::json!({ "text": denied.stderr }),
        )?;
        return write_sse_event(
            stream,
            "done",
            &serde_json::json!({
                "ok": denied.ok,
                "code": denied.code,
                "policy": denied.policy,
                "suggested_args": denied.suggested_args,
            }),
        );
    }
    write_http_stream_header(stream)?;
    write_sse_event(
        stream,
        "start",
        &serde_json::json!({ "args": args, "summary": "started" }),
    )?;
    let mut command = Command::new(env::current_exe().map_err(|error| error.to_string())?);
    command
        .args(args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    apply_env_overlay(&mut command);
    let mut child = command
        .spawn()
        .map_err(|error| format!("local app command failed to start: {error}"))?;
    let stdout = child
        .stdout
        .take()
        .ok_or_else(|| "local app command stdout unavailable".to_string())?;
    let stderr = child
        .stderr
        .take()
        .ok_or_else(|| "local app command stderr unavailable".to_string())?;
    let (sender, receiver) = mpsc::channel::<(String, String)>();
    spawn_stream_reader("stdout", stdout, sender.clone());
    spawn_stream_reader("stderr", stderr, sender);
    loop {
        match receiver.recv_timeout(Duration::from_millis(100)) {
            Ok((kind, text)) => {
                write_sse_event(stream, &kind, &serde_json::json!({ "text": text }))?;
            }
            Err(mpsc::RecvTimeoutError::Timeout) => {
                if child
                    .try_wait()
                    .map_err(|error| error.to_string())?
                    .is_some()
                {
                    break;
                }
            }
            Err(mpsc::RecvTimeoutError::Disconnected) => {
                if child
                    .try_wait()
                    .map_err(|error| error.to_string())?
                    .is_some()
                {
                    break;
                }
                thread::sleep(Duration::from_millis(100));
            }
        }
    }
    let status = child.wait().map_err(|error| error.to_string())?;
    while let Ok((kind, text)) = receiver.try_recv() {
        write_sse_event(stream, &kind, &serde_json::json!({ "text": text }))?;
    }
    write_sse_event(
        stream,
        "done",
        &serde_json::json!({
            "ok": status.success(),
            "code": status.code(),
        }),
    )
}

fn spawn_stream_reader<R>(kind: &str, reader: R, sender: mpsc::Sender<(String, String)>)
where
    R: Read + Send + 'static,
{
    let kind = kind.to_string();
    thread::spawn(move || {
        for text in BufReader::new(reader).lines().map_while(Result::ok) {
            let _ = sender.send((kind.clone(), text));
        }
    });
}

pub(crate) fn denied_response(args: &[String]) -> RunResponse {
    let command = command_name(args).unwrap_or("unknown");
    let state = state_arg(args).unwrap_or_else(|| ".octopus/state.json".to_string());
    let message = format!(
        "local app input is limited to brain-goal. `{command}` is internal, developer-only, or observation-only from this surface. Use chat, goal set/refine, brain --goal, or first-run."
    );
    RunResponse {
        ok: false,
        code: None,
        stdout: String::new(),
        stderr: message,
        policy: Some("user_writes_brain_goal_only".to_string()),
        suggested_args: vec![
            vec![
                "--state".to_string(),
                state.clone(),
                "--json".to_string(),
                "chat".to_string(),
                "describe or refine the goal".to_string(),
            ],
            vec![
                "--state".to_string(),
                state.clone(),
                "--json".to_string(),
                "goal".to_string(),
                "refine".to_string(),
                "tighten the current objective".to_string(),
            ],
            vec![
                "--state".to_string(),
                state,
                "--json".to_string(),
                "brain".to_string(),
                "--goal".to_string(),
                "--save".to_string(),
                "tighten the current objective".to_string(),
            ],
        ],
    }
}

pub(crate) fn apply_env_overlay(command: &mut Command) {
    for (key, value) in env_overlay() {
        command.env(key, value);
    }
}

fn env_overlay() -> Vec<(String, String)> {
    fs::read_to_string(DEFAULT_PROVIDER_ENV_PATH)
        .map(|content| parse_env_overlay(&content))
        .unwrap_or_default()
}

pub(crate) fn parse_env_overlay(content: &str) -> Vec<(String, String)> {
    content.lines().filter_map(parse_env_line).collect()
}

fn parse_env_line(line: &str) -> Option<(String, String)> {
    let line = line.trim();
    if line.is_empty() || line.starts_with('#') {
        return None;
    }
    let assignment = line.strip_prefix("export ").unwrap_or(line);
    let (key, value) = assignment.split_once('=')?;
    let key = key.trim();
    if !key.starts_with("OCTOPUS_") || !valid_env_prefix(key) {
        return None;
    }
    Some((key.to_string(), env_value(value.trim())))
}

fn env_value(value: &str) -> String {
    if let Some(inner) = value
        .strip_prefix('"')
        .and_then(|item| item.strip_suffix('"'))
    {
        if let Some(name) = inner
            .strip_prefix("${")
            .and_then(|item| item.strip_suffix(":-}"))
            .filter(|name| valid_env_prefix(name))
        {
            return env::var(name).unwrap_or_default();
        }
        return inner.replace("\\\"", "\"").replace("\\\\", "\\");
    }
    if let Some(inner) = value
        .strip_prefix('\'')
        .and_then(|item| item.strip_suffix('\''))
    {
        return inner.replace("'\\''", "'");
    }
    value.to_string()
}

pub(crate) fn command_allowed(args: &[String]) -> bool {
    let Some(command) = command_name(args) else {
        return false;
    };
    if command == "provider" {
        return provider_observe_allowed(args);
    }
    if command == "preflight" {
        return preflight_observe_allowed(args);
    }
    if command == "first-run" {
        return first_run_allowed(args);
    }
    if command == "update" {
        return update_allowed(args);
    }
    if command == "brain" {
        return brain_goal_allowed(args);
    }
    if command == "goal" {
        return goal_allowed(args);
    }
    if command == "pet" {
        return pet_observe_allowed(args);
    }
    if command == "starter" {
        return starter_observe_allowed(args);
    }
    if command == "needs" || command == "context" {
        return observe_only(args);
    }
    matches!(
        command,
        "chat"
            | "doctor"
            | "report"
            | "status"
            | "installed"
            | "skills"
            | "catalog"
            | "manifests"
            | "providers"
            | "traces"
    )
}

fn observe_only(args: &[String]) -> bool {
    let Some(index) = command_index(args) else {
        return false;
    };
    args.len() == index + 1
}

fn goal_allowed(args: &[String]) -> bool {
    let Some(index) = command_index(args) else {
        return false;
    };
    matches!(
        args.get(index + 1).map(String::as_str),
        Some("set" | "refine")
    )
}

fn brain_goal_allowed(args: &[String]) -> bool {
    let Some(index) = command_index(args) else {
        return false;
    };
    let rest = &args[index + 1..];
    let mut has_goal = false;
    let mut position = 0;
    while position < rest.len() {
        match rest[position].as_str() {
            "--goal" => has_goal = true,
            "--live" | "--save" | "--session" => {}
            "--apply" | "--apply-json" | "--llm-prefix" | "--provider-prefix" => {
                position += 1;
                if position >= rest.len() {
                    return false;
                }
            }
            "--intent" | "--brief" | "--clarify" | "--agenda" | "--scout" | "--deliberate"
            | "--synthesize" | "--council" | "--reflect" | "--align" | "--memory" | "--focus"
            | "--rewrite" | "--models" => return false,
            value if value.starts_with('-') => return false,
            _ => {}
        }
        position += 1;
    }
    has_goal
}

fn first_run_allowed(args: &[String]) -> bool {
    let Some(index) = command_index(args) else {
        return false;
    };
    args[index + 1..]
        .iter()
        .all(|arg| !arg.starts_with('-') || arg == "--live")
}

fn update_allowed(args: &[String]) -> bool {
    let Some(index) = command_index(args) else {
        return false;
    };
    args.len() == index + 1
        || (args.len() == index + 2
            && args
                .get(index + 1)
                .is_some_and(|value| value == "--dry-run"))
}

fn pet_observe_allowed(args: &[String]) -> bool {
    let Some(index) = command_index(args) else {
        return false;
    };
    let rest = &args[index + 1..];
    rest.len() <= 1 && rest.first().is_none_or(|value| value != "image")
}

fn preflight_observe_allowed(args: &[String]) -> bool {
    let Some(index) = command_index(args) else {
        return false;
    };
    let rest = &args[index + 1..];
    rest.is_empty() || (rest.len() == 1 && rest[0] == "--live")
}

fn starter_observe_allowed(args: &[String]) -> bool {
    let Some(index) = command_index(args) else {
        return false;
    };
    args.get(index + 1)
        .is_none_or(|value| value.as_str() != "feedback")
}

fn provider_observe_allowed(args: &[String]) -> bool {
    let Some(index) = command_index(args) else {
        return false;
    };
    match args.get(index + 1).map(String::as_str) {
        Some("status") => args.len() == index + 2,
        Some("check") => {
            args.get(index + 2)
                .is_none_or(|prefix| valid_env_prefix(prefix))
                && args.len() <= index + 3
        }
        Some(profile) => {
            provider_profile(profile).is_ok()
                && args
                    .get(index + 2)
                    .is_none_or(|prefix| valid_env_prefix(prefix))
                && args.len() <= index + 3
        }
        None => false,
    }
}

pub(crate) fn command_name(args: &[String]) -> Option<&str> {
    command_index(args).and_then(|index| args.get(index).map(String::as_str))
}

fn command_index(args: &[String]) -> Option<usize> {
    let mut index = 0;
    while index < args.len() {
        match args[index].as_str() {
            "--state" | "--lang" => index += 2,
            "--json" => index += 1,
            value if value.starts_with('-') => return None,
            _ => return Some(index),
        }
    }
    None
}

fn state_arg(args: &[String]) -> Option<String> {
    args.windows(2)
        .find(|window| window.first().is_some_and(|arg| arg == "--state"))
        .and_then(|window| window.get(1).cloned())
}
