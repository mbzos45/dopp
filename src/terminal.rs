use log::info;
#[cfg(any(target_os = "linux", target_os = "windows"))]
use log::error;
use std::env;
use std::process::Command;

#[derive(Clone, Copy, Debug)]
enum ContainerRuntime {
    Docker,
    Podman,
}

impl ContainerRuntime {
    fn as_str(self) -> &'static str {
        match self {
            Self::Docker => "docker",
            Self::Podman => "podman",
        }
    }
}

pub struct ExecCommand {
    runtime: ContainerRuntime,
    target: String,
    options: ExecOptions,
}

#[derive(Clone, Copy, Debug)]
pub struct ExecOptions {
    pub inherit_xauthority_on_linux: bool,
}

impl Default for ExecOptions {
    fn default() -> Self {
        Self {
            inherit_xauthority_on_linux: true,
        }
    }
}

impl ExecCommand {
    pub fn new(target: &str, options: ExecOptions) -> Self {
        Self {
            runtime: detect_runtime(),
            target: target.to_string(),
            options,
        }
    }

    pub fn as_string(&self) -> String {
        build_exec_command(
            cfg!(target_os = "linux"),
            self.runtime,
            &self.target,
            &self.options,
            env::var("XAUTHORITY").ok().as_deref(),
        )
    }

    #[cfg(any(target_os = "linux", target_os = "windows"))]
    fn as_tokens(&self) -> Vec<String> {
        build_exec_tokens(
            cfg!(target_os = "linux"),
            self.runtime,
            &self.target,
            &self.options,
            env::var("XAUTHORITY").ok().as_deref(),
        )
    }
}

pub fn launch_exec_terminal(command: &ExecCommand) -> Result<(), String> {
    #[cfg(any(target_os = "linux", target_os = "windows"))]
    let tokens = command.as_tokens();
    #[cfg(target_os = "linux")]
    {
        return launch_linux(&tokens);
    }
    #[cfg(target_os = "windows")]
    {
        return launch_windows(command, &tokens);
    }
    #[cfg(target_os = "macos")]
    {
        return launch_macos(command);
    }
    #[allow(unreachable_code)]
    Err("Unsupported platform for terminal launch".to_string())
}

pub fn copy_to_clipboard(text: &str) -> Result<(), String> {
    let mut clipboard = arboard::Clipboard::new().map_err(|err| err.to_string())?;
    clipboard.set_text(text.to_string()).map_err(|err| err.to_string())
}

fn build_exec_command(
    platform_is_linux: bool,
    runtime: ContainerRuntime,
    target: &str,
    options: &ExecOptions,
    xauthority: Option<&str>,
) -> String {
    build_exec_tokens(platform_is_linux, runtime, target, options, xauthority)
        .into_iter()
        .map(|token| quote_token(&token))
        .collect::<Vec<_>>()
        .join(" ")
}

fn build_exec_tokens(
    platform_is_linux: bool,
    runtime: ContainerRuntime,
    target: &str,
    options: &ExecOptions,
    xauthority: Option<&str>,
) -> Vec<String> {
    let mut tokens = vec![
        runtime.as_str().to_string(),
        "exec".to_string(),
        "-it".to_string(),
    ];

    if let Some(value) = xauthority_env_assignment(platform_is_linux, options, xauthority) {
        tokens.push("-e".to_string());
        tokens.push(format!("XAUTHORITY={value}"));
    }

    tokens.push(target.to_string());
    tokens.push("/bin/bash".to_string());
    tokens
}

fn xauthority_env_assignment(
    platform_is_linux: bool,
    options: &ExecOptions,
    xauthority: Option<&str>,
) -> Option<String> {
    if !platform_is_linux || !options.inherit_xauthority_on_linux {
        return None;
    }

    let value = xauthority?.trim();
    if value.is_empty() {
        None
    } else {
        Some(value.to_string())
    }
}

fn quote_token(token: &str) -> String {
    if token
        .chars()
        .any(|character| character.is_whitespace() || character == '"' || character == '\'')
    {
        format!("{token:?}")
    } else {
        token.to_string()
    }
}

fn detect_runtime() -> ContainerRuntime {
    if env::var_os("PODMAN_HOST").is_some() || env::var_os("PODMAN_CONNECTION").is_some() {
        ContainerRuntime::Podman
    } else {
        ContainerRuntime::Docker
    }
}

#[cfg(any(target_os = "linux", target_os = "windows"))]
fn spawn_command(program: &str, args: &[String]) -> Result<(), String> {
    Command::new(program)
        .args(args)
        .spawn()
        .map(|_| ())
        .map_err(|err| format!("{program} failed: {err}"))
}

#[cfg(target_os = "linux")]
fn launch_linux(tokens: &[String]) -> Result<(), String> {
    let mut attempts: Vec<(String, Vec<String>)> = Vec::new();

    if let Some(terminal) = env::var_os("TERMINAL").and_then(|value| value.into_string().ok()) {
        attempts.push((terminal, prepend_arg("-e", tokens)));
    }

    attempts.push(("xdg-terminal-exec".to_string(), tokens.to_vec()));
    attempts.push((
        "x-terminal-emulator".to_string(),
        prepend_arg("-e", tokens),
    ));

    let mut fallback: Vec<(String, Vec<String>)> = vec![
        ("gnome-terminal".to_string(), prepend_arg("--", tokens)),
        ("konsole".to_string(), prepend_arg("-e", tokens)),
        ("xterm".to_string(), prepend_arg("-e", tokens)),
    ];

    attempts.append(&mut fallback);

    let mut last_error: Option<String> = None;
    for (program, args) in attempts {
        match spawn_command(&program, &args) {
            Ok(()) => {
                info!("launched terminal via {program}");
                return Ok(());
            }
            Err(err) => {
                error!("terminal launch failed via {program}: {err}");
                last_error = Some(err);
            }
        }
    }

    Err(last_error.unwrap_or_else(|| "No terminal available".to_string()))
}

#[cfg(target_os = "windows")]
fn launch_windows(command: &ExecCommand, tokens: &[String]) -> Result<(), String> {
    let exec_string = command.as_string();
    let mut last_error: Option<String> = None;

    let start_args = vec![
        "/C".to_string(),
        "start".to_string(),
        "".to_string(),
        exec_string.clone(),
    ];
    match spawn_command("cmd", &start_args) {
        Ok(()) => {
            info!("launched terminal via start");
            return Ok(());
        }
        Err(err) => {
            error!("terminal launch failed via start: {err}");
            last_error = Some(err);
        }
    }

    let mut wt_args = vec![
        "-w".to_string(),
        "0".to_string(),
        "new-tab".to_string(),
        "--".to_string(),
    ];
    wt_args.extend(tokens.iter().cloned());
    match spawn_command("wt.exe", &wt_args) {
        Ok(()) => {
            info!("launched terminal via wt.exe");
            return Ok(());
        }
        Err(err) => {
            error!("terminal launch failed via wt.exe: {err}");
            last_error = Some(err);
        }
    }

    let cmd_args = vec!["/K".to_string(), exec_string];
    match spawn_command("cmd.exe", &cmd_args) {
        Ok(()) => {
            info!("launched terminal via cmd.exe");
            Ok(())
        }
        Err(err) => {
            error!("terminal launch failed via cmd.exe: {err}");
            Err(last_error.unwrap_or(err))
        }
    }
}

#[cfg(target_os = "macos")]
fn launch_macos(command: &ExecCommand) -> Result<(), String> {
    let exec_string = command.as_string();
    let iterm_script = format!(
        "tell application \"iTerm2\"\n  activate\n  set newWindow to (create window with default profile)\n  tell current session of newWindow to write text \"{}\"\nend tell",
        escape_applescript_string(&exec_string)
    );
    if run_osascript(&iterm_script) {
        info!("launched terminal via iTerm2");
        return Ok(());
    }

    let terminal_script = format!(
        "tell application \"Terminal\"\n  activate\n  do script \"{}\"\nend tell",
        escape_applescript_string(&exec_string)
    );
    if run_osascript(&terminal_script) {
        info!("launched terminal via Terminal.app");
        return Ok(());
    }

    Err("AppleScript launch failed for iTerm2 and Terminal.app".to_string())
}

#[cfg(target_os = "macos")]
fn run_osascript(script: &str) -> bool {
    Command::new("osascript")
        .args(["-e", script])
        .status()
        .map(|status| status.success())
        .unwrap_or(false)
}

#[cfg(target_os = "macos")]
fn escape_applescript_string(input: &str) -> String {
    input.replace('\\', "\\\\").replace('"', "\\\"")
}

#[cfg(any(target_os = "linux", target_os = "windows"))]
fn prepend_arg(arg: &str, tokens: &[String]) -> Vec<String> {
    let mut args = Vec::with_capacity(tokens.len() + 1);
    args.push(arg.to_string());
    args.extend(tokens.iter().cloned());
    args
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn xauthority_is_included_on_linux_when_enabled() {
        let options = ExecOptions::default();

        let value = xauthority_env_assignment(true, &options, Some("/tmp/.Xauthority"));

        assert_eq!(value, Some("/tmp/.Xauthority".to_string()));
    }

    #[test]
    fn xauthority_is_skipped_when_missing_or_disabled() {
        let enabled = ExecOptions::default();
        let disabled = ExecOptions {
            inherit_xauthority_on_linux: false,
        };

        assert_eq!(xauthority_env_assignment(true, &enabled, None), None);
        assert_eq!(xauthority_env_assignment(true, &enabled, Some("   ")), None);
        assert_eq!(xauthority_env_assignment(false, &enabled, Some("/tmp/.Xauthority")), None);
        assert_eq!(xauthority_env_assignment(true, &disabled, Some("/tmp/.Xauthority")), None);
    }

    #[test]
    fn build_exec_command_includes_xauthority_only_when_requested() {
        let options = ExecOptions::default();

        let command = build_exec_command(
            true,
            ContainerRuntime::Docker,
            "my-container",
            &options,
            Some("/tmp/.Xauthority"),
        );

        assert!(command.contains("-e XAUTHORITY=/tmp/.Xauthority"));

        let command_without_xauthority = build_exec_command(
            false,
            ContainerRuntime::Docker,
            "my-container",
            &options,
            Some("/tmp/.Xauthority"),
        );

        assert!(!command_without_xauthority.contains("-e XAUTHORITY="));
    }
}
