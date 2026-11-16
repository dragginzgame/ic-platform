//! Private server supervisor. Host Process owns spawning, signalling and reaping.
use ic_host_process::{Input, Ownership, Policy, Stderr, Stream};
use std::{
    io::{self, Write},
    process::Command,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    thread::{self, JoinHandle},
    time::{Duration, Instant},
};

struct Server {
    cancel: Arc<AtomicBool>,
    worker: Option<JoinHandle<ic_host_process::Report>>,
}
impl Server {
    fn close(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.cancel.store(true, Ordering::Relaxed);
        let Some(worker) = self.worker.take() else {
            return Ok(());
        };
        let report = worker.join().map_err(|_| "server supervisor panicked")?;
        if !report.cleanup.is_empty()
            || !matches!(report.failure, Some(ic_host_process::Failure::Cancelled))
        {
            return Err(format!("server termination: {report:?}").into());
        }
        Ok(())
    }
}
impl Drop for Server {
    fn drop(&mut self) {
        if let Err(error) = self.close() {
            eprintln!("server cleanup failed: {error}");
        }
    }
}
fn policy(seconds: u64) -> Policy {
    Policy {
        ownership: Ownership::Group,
        input: Input::Null,
        stdout_bytes: 64 * 1024 * 1024,
        stderr: Stderr::Inherit,
        elapsed: Duration::from_secs(seconds),
    }
}
fn output(_stream: Stream, bytes: &[u8]) -> io::Result<()> {
    io::stdout().write_all(bytes)
}
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = std::env::args_os().skip(1);
    let binary = args.next().ok_or("expected authenticated server binary")?;
    let program = args.next().ok_or("expected qualification command")?;
    let directory = tempfile::tempdir()?;
    let port_file = directory.path().join("port");
    let cancel = Arc::new(AtomicBool::new(false));
    let signal = Arc::clone(&cancel);
    let mut command = Command::new(binary);
    command
        .args(["--hard-ttl", "600", "--port-file"])
        .arg(&port_file);
    let worker =
        thread::spawn(move || ic_host_process::run(&mut command, policy(610), &signal, output));
    let mut server = Server {
        cancel,
        worker: Some(worker),
    };
    let deadline = Instant::now() + Duration::from_secs(30);
    let port = loop {
        if server.worker.as_ref().is_some_and(JoinHandle::is_finished) {
            return Err("server exited before readiness".into());
        }
        if let Ok(text) = std::fs::read_to_string(&port_file)
            && text.ends_with('\n')
        {
            let port: u16 = text.trim().parse()?;
            if port == 0 {
                return Err("server returned port zero".into());
            }
            break port;
        }
        if Instant::now() >= deadline {
            return Err("server readiness deadline expired".into());
        }
        thread::sleep(Duration::from_millis(20));
    };
    let mut child = Command::new(program);
    child
        .args(args)
        .env("POCKET_IC_SERVER_URL", format!("http://127.0.0.1:{port}/"));
    let report = ic_host_process::run(&mut child, policy(570), &AtomicBool::new(false), output);
    let cleanup = server.close();
    if report.failure.is_some()
        || !report.cleanup.is_empty()
        || !report.status.is_some_and(|s| s.success())
    {
        return Err(
            format!("qualification command failed: {report:?}; server close: {cleanup:?}").into(),
        );
    }
    cleanup
}
