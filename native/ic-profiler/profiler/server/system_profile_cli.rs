//! Online, offline, and host-attachment access to the same profiling queries.
use clap::{Args, ValueEnum};
use ic_system_profiler::{RunConfig, StopOptions};
use serde_json::{Value, json};
use std::{
    collections::HashMap,
    ffi::OsString,
    io::Write,
    path::PathBuf,
    process::Command,
    time::{Duration, Instant},
};

#[derive(Clone, Debug, ValueEnum)]
pub enum ProfileAction {
    Start,
    Stop,
    Delete,
    List,
    Status,
    Summary,
    Components,
    Resources,
    Metrics,
    Events,
    Flows,
    Limits,
    Rounds,
    Pricing,
    Explain,
    Compare,
    Export,
    Import,
    Render,
    Capture,
    Record,
}

#[derive(Debug, Args)]
pub struct ProfileArgs {
    #[arg(value_enum)]
    action: ProfileAction,
    /// PocketIC API or viewer base URL. Omit for offline --input or capture/record.
    #[arg(long)]
    server: Option<String>,
    #[arg(long)]
    run: Option<String>,
    #[arg(long)]
    other_run: Option<String>,
    #[arg(long)]
    name: Option<String>,
    /// Descriptive backend label; defaults to pocketic online, attached_process locally.
    #[arg(long)]
    backend: Option<String>,
    /// Root process to observe; independently launched services use --extra-pid.
    #[arg(long)]
    pid: Option<u32>,
    #[arg(long)]
    extra_pid: Vec<u32>,
    #[arg(long, default_value_t = 100)]
    sample_interval_ms: u64,
    /// Existing IC metrics: standard keeps scalar/count/sum; full adds bounded raw buckets; none skips gathering.
    #[arg(long,default_value="standard",value_parser=["none","standard","full"])]
    metrics_level: String,
    /// PSS detail interval; zero disables all PSS scans, including boundaries.
    #[arg(long, default_value_t = 1000)]
    memory_detail_interval_ms: u64,
    #[arg(long, default_value_t = 10.0)]
    duration: f64,
    #[arg(long, default_value_t = 0)]
    drain_ms: u64,
    #[arg(long)]
    input: Option<PathBuf>,
    #[arg(long)]
    other_input: Option<PathBuf>,
    #[arg(long)]
    output: Option<PathBuf>,
    #[arg(long, default_value = "components")]
    view: String,
    /// JSON object of bounded query parameters, e.g. {"limit":100,"component":"ic.execution"}.
    #[arg(long, default_value = "{}")]
    query: String,
    /// PNG, SVG, or JSON image evidence sidecar.
    #[arg(long, default_value = "svg")]
    format: String,
    /// Capture a command and descendants. Command arguments follow --.
    #[arg(last = true)]
    command: Vec<OsString>,
    /// Keep a launched command running after the capture window.
    #[arg(long)]
    keep_command_running: bool,
}

fn id(value: &Value) -> Result<String, String> {
    value
        .get("run_id")
        .and_then(Value::as_str)
        .map(str::to_owned)
        .ok_or("Profiler did not return run_id".into())
}
fn output(bytes: &[u8], path: Option<&PathBuf>) -> Result<(), String> {
    if let Some(path) = path {
        std::fs::write(path, bytes).map_err(|e| format!("{}: {e}", path.display()))
    } else {
        std::io::stdout()
            .lock()
            .write_all(bytes)
            .map_err(|e| e.to_string())
    }
}
fn json_output(value: &Value, path: Option<&PathBuf>) -> Result<(), String> {
    let mut bytes = serde_json::to_vec_pretty(value).map_err(|e| e.to_string())?;
    bytes.push(b'\n');
    output(&bytes, path)
}

pub fn run(args: &ProfileArgs) -> Result<(), String> {
    let query: Value =
        serde_json::from_str(&args.query).map_err(|e| format!("Invalid --query JSON: {e}"))?;
    query.as_object().ok_or("--query must be an object")?;
    let mut parameters = crate::system_profile_api::argument_parameters(&query);
    if !args.duration.is_finite() || args.duration < 0.0 || args.duration > 86400.0 {
        return Err("--duration must be between 0 and 86400 seconds".into());
    }
    if matches!(args.action, ProfileAction::Capture | ProfileAction::Record) {
        return record(args);
    }
    if let Some(server) = &args.server {
        return remote(args, server, &parameters);
    }
    let imported = args
        .input
        .as_ref()
        .map(|path| {
            std::fs::read(path)
                .map_err(|e| e.to_string())
                .and_then(|b| crate::system_profile_api::import_bytes(&b))
        })
        .transpose()?;
    let run_id = args
        .run
        .clone()
        .or_else(|| imported.as_ref().and_then(|v| id(v).ok()));
    let require_run = || {
        run_id
            .as_deref()
            .ok_or("Pass --run, --server, or --input as appropriate".to_string())
    };
    match args.action {
        ProfileAction::Explain => json_output(&ic_system_profiler::explain(), args.output.as_ref()),
        ProfileAction::List => json_output(&ic_system_profiler::list_runs(), args.output.as_ref()),
        ProfileAction::Import => json_output(
            imported.as_ref().ok_or("Pass --input bundle.icprofile")?,
            args.output.as_ref(),
        ),
        ProfileAction::Export => output(
            &crate::system_profile_api::export_bytes(require_run()?)?,
            args.output.as_ref(),
        ),
        ProfileAction::Compare => {
            let other = args
                .other_input
                .as_ref()
                .map(|path| {
                    std::fs::read(path)
                        .map_err(|e| e.to_string())
                        .and_then(|b| crate::system_profile_api::import_bytes(&b))
                })
                .transpose()?;
            let b = args
                .other_run
                .clone()
                .or_else(|| other.as_ref().and_then(|v| id(v).ok()))
                .ok_or("Pass --other-run or --other-input")?;
            json_output(
                &ic_system_profiler::compare(require_run()?, &b)?,
                args.output.as_ref(),
            )
        }
        ProfileAction::Render => {
            parameters.insert("view".into(), args.view.clone());
            let rendered = crate::system_profile_api::render(require_run()?, &parameters)?;
            match args.format.as_str() {
                "svg" => output(rendered.svg.as_bytes(), args.output.as_ref()),
                "png" => output(&rendered.png, args.output.as_ref()),
                "json" => json_output(&rendered.metadata, args.output.as_ref()),
                _ => Err("--format must be svg, png, or json".into()),
            }
        }
        ProfileAction::Start | ProfileAction::Stop => Err(
            "Use --server for start/stop, or capture/record for a self-contained capture".into(),
        ),
        ProfileAction::Delete => crate::system_profile_api::dispatch_sync(
            None,
            "profile_delete",
            &json!({"run_id":require_run()?}),
        )
        .and_then(|value| json_output(&value, args.output.as_ref())),
        _ => json_output(
            &crate::system_profile_api::query_run(
                require_run()?,
                action_view(&args.action),
                &parameters,
            )?,
            args.output.as_ref(),
        ),
    }
}

fn action_view(action: &ProfileAction) -> &'static str {
    match action {
        ProfileAction::Status => "status",
        ProfileAction::Summary => "summary",
        ProfileAction::Components => "components",
        ProfileAction::Resources => "resources",
        ProfileAction::Metrics => "metrics",
        ProfileAction::Events => "events",
        ProfileAction::Flows => "flows",
        ProfileAction::Limits => "limits",
        ProfileAction::Rounds => "rounds",
        ProfileAction::Pricing => "pricing",
        _ => "summary",
    }
}

fn capture_config(args: &ProfileArgs, default_name: String, default_backend: &str) -> RunConfig {
    RunConfig {
        name: args.name.clone().unwrap_or(default_name),
        root_pid: args.pid,
        extra_pids: args.extra_pid.clone(),
        sample_interval_ms: args.sample_interval_ms,
        metrics_level: args.metrics_level.clone(),
        memory_detail_interval_ms: args.memory_detail_interval_ms,
        backend: args
            .backend
            .clone()
            .unwrap_or_else(|| default_backend.into()),
        ..Default::default()
    }
}

fn start_parameters(args: &ProfileArgs) -> Value {
    let config = capture_config(args, "system profile".into(), "pocketic");
    // Preserve the existing remote-start field set; local-only metadata and
    // other RunConfig defaults are not added to the HTTP request.
    json!({"name":config.name,"root_pid":config.root_pid,"extra_pids":config.extra_pids,
        "sample_interval_ms":config.sample_interval_ms,"backend":config.backend,
        "metrics_level":config.metrics_level,"memory_detail_interval_ms":config.memory_detail_interval_ms})
}

fn remote(
    args: &ProfileArgs,
    server: &str,
    parameters: &HashMap<String, String>,
) -> Result<(), String> {
    let base = reqwest::Url::parse(&format!("{}/", server.trim_end_matches('/')))
        .map_err(|e| e.to_string())?;
    let client = reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(300))
        .build()
        .map_err(|e| e.to_string())?;
    let run = args.run.as_deref();
    let require_run = || run.ok_or("Pass --run RUN_ID");
    let path = match args.action {
        ProfileAction::List | ProfileAction::Start => "api/profiling/runs".to_string(),
        ProfileAction::Explain if run.is_none() => "api/profiling/explain".to_string(),
        ProfileAction::Import => "api/profiling/import".to_string(),
        ProfileAction::Compare => "api/profiling/compare".to_string(),
        ProfileAction::Stop => format!("api/profiling/runs/{}/stop", require_run()?),
        ProfileAction::Delete => format!("api/profiling/runs/{}", require_run()?),
        ProfileAction::Export => format!("api/profiling/runs/{}/export", require_run()?),
        ProfileAction::Render => {
            if !["svg", "png", "json"].contains(&args.format.as_str()) {
                return Err("--format must be svg, png, or json".into());
            }
            format!(
                "api/profiling/runs/{}/render.{}",
                require_run()?,
                args.format
            )
        }
        ProfileAction::Explain => format!("api/profiling/runs/{}/explain", require_run()?),
        _ => format!(
            "api/profiling/runs/{}/{}",
            require_run()?,
            action_view(&args.action)
        ),
    };
    let mut url = base.join(&path).map_err(|e| e.to_string())?;
    {
        let mut pairs = url.query_pairs_mut();
        for (key, value) in parameters {
            pairs.append_pair(key, value);
        }
        if matches!(args.action, ProfileAction::Compare) {
            pairs.append_pair("a", require_run()?);
            pairs.append_pair("b", args.other_run.as_deref().ok_or("Pass --other-run")?);
        }
        if matches!(args.action, ProfileAction::Render) {
            pairs.append_pair("view", &args.view);
        }
    }
    let request = match args.action {
        ProfileAction::Start => client.post(url).json(&start_parameters(args)),
        ProfileAction::Stop => client.post(url).json(&json!({"drain_ms":args.drain_ms})),
        ProfileAction::Delete => client.delete(url),
        ProfileAction::Import => client.post(url).body(
            std::fs::read(args.input.as_ref().ok_or("Pass --input")?).map_err(|e| e.to_string())?,
        ),
        _ => client.get(url),
    };
    let response = request.send().map_err(|e| e.to_string())?;
    let status = response.status();
    let bytes = response.bytes().map_err(|e| e.to_string())?;
    if !status.is_success() {
        return Err(format!(
            "HTTP {status}: {}",
            String::from_utf8_lossy(&bytes)
        ));
    }
    if matches!(args.action, ProfileAction::Export | ProfileAction::Render) {
        output(&bytes, args.output.as_ref())
    } else {
        json_output(
            &serde_json::from_slice(&bytes).map_err(|e| format!("Invalid profile JSON: {e}"))?,
            args.output.as_ref(),
        )
    }
}

struct OwnedCommand {
    child: std::process::Child,
    keep_running: bool,
}

impl Drop for OwnedCommand {
    fn drop(&mut self) {
        if self.keep_running {
            let _ = self.child.try_wait();
            return;
        }
        #[cfg(unix)]
        {
            // The command is placed in its own process group before exec.
            // This never signals an attached process or an unrelated server.
            unsafe {
                libc::kill(-(self.child.id() as i32), libc::SIGTERM);
            }
            let deadline = Instant::now() + Duration::from_millis(500);
            while Instant::now() < deadline {
                if child_finished(&mut self.child).ok().flatten().is_some() {
                    break;
                }
                std::thread::sleep(Duration::from_millis(10));
            }
            unsafe {
                libc::kill(-(self.child.id() as i32), libc::SIGKILL);
            }
        }
        #[cfg(not(unix))]
        {
            let _ = self.child.kill();
        }
        let _ = self.child.wait();
    }
}

fn child_finished(child: &mut std::process::Child) -> Result<Option<String>, String> {
    #[cfg(target_os = "linux")]
    {
        // Keep the zombie's /proc counters available for the final sample.
        let mut info: libc::siginfo_t = unsafe { std::mem::zeroed() };
        let result = unsafe {
            libc::waitid(
                libc::P_PID,
                child.id(),
                &mut info,
                libc::WEXITED | libc::WNOHANG | libc::WNOWAIT,
            )
        };
        if result != 0 {
            return Err(std::io::Error::last_os_error().to_string());
        }
        return Ok((unsafe { info.si_pid() } != 0).then(|| {
            format!(
                "wait status {} (code {})",
                unsafe { info.si_status() },
                info.si_code
            )
        }));
    }
    #[cfg(not(target_os = "linux"))]
    {
        child
            .try_wait()
            .map(|status| status.map(|status| status.to_string()))
            .map_err(|e| e.to_string())
    }
}

/// Exec a constant launcher first so Command::spawn can return its PID, but
/// prevent the workload from running until its profiling baseline is ready.
/// Blocking in pre_exec itself would deadlock Rust's spawn/exec handshake.
fn workload_command(arguments: &[OsString]) -> Result<(Command, Option<std::fs::File>), String> {
    let program = arguments
        .first()
        .ok_or("record requires -- command [args...]")?;
    #[cfg(unix)]
    {
        use std::os::fd::{AsRawFd, FromRawFd, OwnedFd};
        use std::os::unix::process::CommandExt;

        let mut descriptors = [-1; 2];
        // SAFETY: descriptors has space for both pipe output descriptors.
        if unsafe { libc::pipe(descriptors.as_mut_ptr()) } != 0 {
            return Err(format!(
                "Unable to create workload start gate: {}",
                std::io::Error::last_os_error()
            ));
        }
        // SAFETY: pipe succeeded and transfers ownership of two unique FDs.
        let read = unsafe { OwnedFd::from_raw_fd(descriptors[0]) };
        let write = unsafe { OwnedFd::from_raw_fd(descriptors[1]) };
        for fd in [read.as_raw_fd(), write.as_raw_fd()] {
            // SAFETY: fcntl operates on live descriptors and retains no pointers.
            if unsafe { libc::fcntl(fd, libc::F_SETFD, libc::FD_CLOEXEC) } < 0 {
                return Err(std::io::Error::last_os_error().to_string());
            }
        }
        let write_fd = write.as_raw_fd();
        let mut command = Command::new("/bin/sh");
        command
            .args([
                "-c",
                "read -r _ <&3 || exit 125; exec 3<&-; exec \"$@\"",
                "ic-profile-workload",
            ])
            .args(arguments);
        // SAFETY: the post-fork closure only invokes async-signal-safe FD
        // operations. It never waits; the exec'ed shell owns the actual gate.
        unsafe {
            command.pre_exec(move || {
                let source = read.as_raw_fd();
                if libc::dup2(source, 3) < 0 || libc::fcntl(3, libc::F_SETFD, 0) < 0 {
                    return Err(std::io::Error::last_os_error());
                }
                if source != 3 {
                    libc::close(source);
                }
                if write_fd != 3 {
                    libc::close(write_fd);
                }
                Ok(())
            });
        }
        let _ = program; // argv is passed without interpolation through "$@".
        Ok((command, Some(std::fs::File::from(write))))
    }
    #[cfg(not(unix))]
    {
        let mut command = Command::new(program);
        command.args(&arguments[1..]);
        Ok((command, None))
    }
}

fn record(args: &ProfileArgs) -> Result<(), String> {
    ic_system_profiler::initialize_transport()?;
    let mut launch_gate = None;
    let mut child = if matches!(args.action, ProfileAction::Record) {
        let (mut command, gate) = workload_command(&args.command)?;
        launch_gate = gate;
        command.envs(ic_system_profiler::transport_environment());
        // Keep the profiler's stdout machine-readable even when the workload
        // logs to stdout. Its output follows stderr; the command's argv and
        // environment remain unchanged.
        #[cfg(unix)]
        {
            use std::os::fd::AsFd;
            command.stdout(
                std::io::stderr()
                    .as_fd()
                    .try_clone_to_owned()
                    .map_err(|e| format!("Unable to duplicate workload log destination: {e}"))?,
            );
        }
        #[cfg(windows)]
        {
            use std::os::windows::io::AsHandle;
            command.stdout(
                std::io::stderr()
                    .as_handle()
                    .try_clone_to_owned()
                    .map_err(|e| format!("Unable to duplicate workload log destination: {e}"))?,
            );
        }
        #[cfg(unix)]
        {
            use std::os::unix::process::CommandExt;
            command.process_group(0);
        }
        Some(OwnedCommand {
            child: command
                .spawn()
                .map_err(|e| format!("Unable to launch capture command: {e}"))?,
            keep_running: args.keep_command_running,
        })
    } else {
        None
    };
    let pid = child
        .as_ref()
        .map(|c| c.child.id())
        .or(args.pid)
        .ok_or("capture requires --pid PID")?;
    let mut config = capture_config(args, format!("process {pid}"), "attached_process");
    config.root_pid = Some(pid);
    config.metadata = json!({"capture_kind":if child.is_some(){"launched_command"}else{"attached_process"},"startup_scope":if launch_gate.is_some(){"workload exec is gated until capture baseline and child instrumentation activation are ready"}else{"capture begins after spawn/attach; earlier activity is not observed"},"collector_outside_target_tree":true,"command_cleanup_outside_capture":true});
    let run = ic_system_profiler::start(config)?;
    let run_id = id(&run)?;
    if let Some(mut gate) = launch_gate {
        gate.write_all(b"start\n")
            .map_err(|e| format!("Unable to release workload start gate: {e}"))?;
        // Closing this parent writer also ensures no gate FD leaks to the
        // workload. Its original stdin remains untouched.
    }
    let deadline = Instant::now() + Duration::from_secs_f64(args.duration);
    let mut exit_status = None;
    loop {
        if let Some(command) = &mut child {
            if let Some(status) = child_finished(&mut command.child)? {
                exit_status = Some(status);
                break;
            }
        }
        if Instant::now() >= deadline {
            break;
        }
        std::thread::sleep(Duration::from_millis(20));
    }
    let mut result = ic_system_profiler::stop(
        &run_id,
        &StopOptions {
            drain_ms: args.drain_ms,
            reason: Some("CLI capture completed".into()),
        },
    )?;
    let command_exited = exit_status.is_some();
    result["command_exit_status"] = json!(exit_status);
    result["command_kept_running"] =
        json!(!command_exited && args.keep_command_running && child.is_some());
    if let Some(path) = &args.output {
        output(
            &crate::system_profile_api::export_bytes(&run_id)?,
            Some(path),
        )?;
        result["bundle"] = json!(path);
    }
    // Cleanup is deliberately outside the measured interval and affects only
    // the command explicitly launched by record, never capture --pid targets.
    drop(child);
    json_output(&result, None)
}

#[cfg(all(test, unix))]
mod tests {
    use super::*;

    #[test]
    fn workload_waits_until_capture_gate_and_preserves_argument_bytes() {
        let arguments = [
            "/bin/sh",
            "-c",
            "printf '%s' \"$1\"",
            "workload",
            "literal $() and spaces",
        ]
        .map(OsString::from);
        let (mut command, mut gate) = workload_command(&arguments).unwrap();
        command.stdout(std::process::Stdio::piped());
        let mut child = command.spawn().unwrap();
        std::thread::sleep(Duration::from_millis(20));
        assert!(
            child.try_wait().unwrap().is_none(),
            "workload ran before the capture baseline"
        );
        gate.as_mut().unwrap().write_all(b"start\n").unwrap();
        drop(gate);
        let result = child.wait_with_output().unwrap();
        assert!(result.status.success());
        assert_eq!(result.stdout, b"literal $() and spaces");
    }
}

#[cfg(test)]
mod adapter_tests {
    use super::*;
    use clap::Parser;
    #[derive(Parser)]
    struct CommandLine {
        #[command(flatten)]
        profile: ProfileArgs,
    }

    #[test]
    fn capture_config_preserves_context_defaults_and_remote_field_set() {
        let args = CommandLine::try_parse_from(["fixture", "start"])
            .unwrap()
            .profile;
        assert_eq!(
            start_parameters(&args),
            json!({"name":"system profile","root_pid":null,"extra_pids":[],"sample_interval_ms":100,"backend":"pocketic","metrics_level":"standard","memory_detail_interval_ms":1000})
        );
        let local = capture_config(&args, "process 42".into(), "attached_process");
        assert_eq!(local.name, "process 42");
        assert_eq!(local.backend, "attached_process");
        let args = CommandLine::try_parse_from([
            "fixture",
            "start",
            "--name",
            "custom",
            "--backend",
            "explicit",
            "--pid",
            "42",
            "--metrics-level",
            "none",
        ])
        .unwrap()
        .profile;
        let local = capture_config(&args, "fallback".into(), "attached_process");
        let remote = start_parameters(&args);
        assert_eq!(local.name, "custom");
        assert_eq!(local.backend, "explicit");
        assert_eq!(local.root_pid, Some(42));
        assert_eq!(remote["root_pid"], 42);
        assert_eq!(remote["metrics_level"], "none");
    }
}
