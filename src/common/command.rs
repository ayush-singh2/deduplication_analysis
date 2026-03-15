//! Safe external command execution — mirrors Python's `execute_command`.

use std::process::{Command, Stdio};
use std::time::Duration;

use tracing::error;

/// Shell metacharacters that are rejected in command arguments.
const UNSAFE_CHARS: &[char] = &[';', '&', '|', '>', '<', '$', '`'];

/// Execute an external command safely.
///
/// Returns `(exit_code, stdout, stderr)`.  An exit code of `-1` indicates the
/// command could not be launched or timed out.
pub fn execute_command(cmd: &[&str], timeout: Option<Duration>) -> (i32, String, String) {
    // Validate arguments for shell metacharacters.
    for arg in cmd {
        if arg.contains(UNSAFE_CHARS) {
            let msg = "Command contains potentially unsafe characters".to_string();
            return (-1, String::new(), msg);
        }
    }

    let program = match cmd.first() {
        Some(p) => *p,
        None => return (-1, String::new(), "Empty command".to_string()),
    };

    let mut child = match Command::new(program)
        .args(&cmd[1..])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
    {
        Ok(c) => c,
        Err(e) => {
            error!("Error executing command {program}: {e}");
            return (-1, String::new(), e.to_string());
        }
    };

    // Apply timeout if specified.
    if let Some(dur) = timeout {
        match child.wait_timeout(dur) {
            Ok(Some(status)) => {
                let stdout = read_pipe(child.stdout.take());
                let stderr = read_pipe(child.stderr.take());
                let code = status.code().unwrap_or(-1);
                return (code, stdout, stderr);
            }
            Ok(None) => {
                // Timed out — kill the process.
                let _ = child.kill();
                let _ = child.wait();
                error!("Command timed out: {}", cmd.join(" "));
                return (-1, String::new(), "Command timed out".to_string());
            }
            Err(e) => {
                let _ = child.kill();
                return (-1, String::new(), e.to_string());
            }
        }
    }

    // No timeout — block until completion.
    match child.wait_with_output() {
        Ok(output) => {
            let code = output.status.code().unwrap_or(-1);
            let stdout = String::from_utf8_lossy(&output.stdout).to_string();
            let stderr = String::from_utf8_lossy(&output.stderr).to_string();
            (code, stdout, stderr)
        }
        Err(e) => {
            error!("Error waiting for command {program}: {e}");
            (-1, String::new(), e.to_string())
        }
    }
}

/// Read a pipe handle into a `String`.
fn read_pipe<R: std::io::Read>(pipe: Option<R>) -> String {
    let mut buf = String::new();
    if let Some(mut r) = pipe {
        let _ = r.read_to_string(&mut buf);
    }
    buf
}

/// Extension trait to add `wait_timeout` to `std::process::Child`,
/// similar to the nix/windows APIs.
trait ChildExt {
    fn wait_timeout(&mut self, dur: Duration) -> std::io::Result<Option<std::process::ExitStatus>>;
}

impl ChildExt for std::process::Child {
    fn wait_timeout(
        &mut self,
        dur: Duration,
    ) -> std::io::Result<Option<std::process::ExitStatus>> {
        let start = std::time::Instant::now();
        let poll_interval = Duration::from_millis(50);

        loop {
            match self.try_wait()? {
                Some(status) => return Ok(Some(status)),
                None => {
                    if start.elapsed() >= dur {
                        return Ok(None);
                    }
                    std::thread::sleep(poll_interval);
                }
            }
        }
    }
}
