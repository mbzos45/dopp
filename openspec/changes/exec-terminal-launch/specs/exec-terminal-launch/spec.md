## ADDED Requirements

### Requirement: Exec button launches terminal session
The system SHALL open a new terminal window connected to the selected container when the exec button is pressed, executing "[docker/podman] exec -it <container-name-or-id> /bin/bash".

#### Scenario: Launch terminal with exec command
- **WHEN** the user clicks exec for a running container
- **THEN** the system opens a new terminal window running the exec command for that container

### Requirement: Linux terminal resolution and fallback
On Linux, the system SHALL resolve the terminal launcher in this order: $TERMINAL -> xdg-terminal-exec -> x-terminal-emulator, and if none are available, fall back in order to gnome-terminal -> konsole -> xterm.

#### Scenario: Resolve Linux terminal
- **WHEN** the user clicks exec on Linux
- **THEN** the system launches the first available terminal in the defined Linux priority order

### Requirement: Windows terminal resolution and fallback
On Windows, the system SHALL use the OS default terminal via the start command, and if that fails, SHALL attempt wt.exe and then cmd.exe.

#### Scenario: Resolve Windows terminal
- **WHEN** the user clicks exec on Windows
- **THEN** the system launches the terminal using start, or the defined Windows fallback order on failure

### Requirement: macOS terminal resolution and fallback
On macOS, the system SHALL launch a terminal via AppleScript, preferring iTerm2 and falling back to Terminal.app.

#### Scenario: Resolve macOS terminal
- **WHEN** the user clicks exec on macOS
- **THEN** the system launches iTerm2 via AppleScript or falls back to Terminal.app

### Requirement: Failure handling and recovery
If the terminal launch fails, the system SHALL show an error to the user and copy the exec command to the clipboard for manual use.

#### Scenario: Terminal launch fails
- **WHEN** the system cannot launch a terminal for exec
- **THEN** the user sees an error and the exec command is copied to the clipboard

### Requirement: Error logging for launch failures
The system SHALL log terminal launch failures with enough detail to diagnose which resolution step failed.

#### Scenario: Log failure details
- **WHEN** a terminal launch attempt fails
- **THEN** the system logs the attempted launcher and the failure detail
