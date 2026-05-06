# dopp

A minimal GUI for basic Docker container control built with `eframe`/`egui`.

## Features

- One-row-per-container layout with action buttons on the left
- State display with color hints and container name
- Refresh button to re-fetch container list
- Window height grows as containers increase
- Exec button is a UI placeholder (no action yet)

## Manual Test

1. Start Docker Desktop (or ensure the Docker daemon is running).
2. Run the app and verify containers are listed.
3. Use **start/stop/restart** buttons and confirm the state updates after refresh.
4. Stop a container and confirm **restart/exec** are hidden for that row.
5. Add more containers and press **refresh**; confirm the window height increases.

## Run

```shell
cargo run
```
