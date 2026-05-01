use anyhow::{anyhow, Context, Result};
use bollard::container::{ListContainersOptions, RestartContainerOptions, StartContainerOptions, StopContainerOptions};
use bollard::{Docker, API_DEFAULT_VERSION};
use log::{error, info, warn};
use serde::Serialize;
use std::env;

const DEFAULT_SOCKET_CANDIDATES: [&str; 4] = [
  "unix:///var/run/docker.sock",
  "unix:///run/podman/podman.sock",
  "unix://$XDG_RUNTIME_DIR/podman/podman.sock",
  "unix://$HOME/.podman/podman.sock",
];

#[derive(Debug, Serialize)]
pub struct ContainerInfo {
  pub id: String,
  pub name: String,
  pub image: String,
  pub state: String,
  pub status: String,
}

#[tauri::command]
pub async fn list_containers() -> Result<Vec<ContainerInfo>, String> {
  let docker = connect_engine().await.map_err(engine_error_message)?;
  let options = Some(ListContainersOptions::<String> {
    all: true,
    ..Default::default()
  });

  let containers = docker
    .list_containers(options)
    .await
    .map_err(|err| {
      error!("list_containers failed: {err:?}");
      engine_error_message(err)
    })?;

  Ok(containers
    .into_iter()
    .map(|container| {
      let name = container
        .names
        .unwrap_or_default()
        .into_iter()
        .next()
        .unwrap_or_default()
        .trim_start_matches('/')
        .to_string();

      ContainerInfo {
        id: container.id.unwrap_or_default(),
        name,
        image: container.image.unwrap_or_default(),
        state: container.state.unwrap_or_else(|| "unknown".to_string()),
        status: container.status.unwrap_or_default(),
      }
    })
    .collect())
}

#[tauri::command]
pub async fn start_container(container_id: String) -> Result<(), String> {
  let docker = connect_engine().await.map_err(engine_error_message)?;
  docker
    .start_container(&container_id, None::<StartContainerOptions<String>>)
    .await
    .map_err(|err| {
      error!("start_container failed: {err:?}");
      "Failed to start container.".to_string()
    })
}

#[tauri::command]
pub async fn stop_container(container_id: String) -> Result<(), String> {
  let docker = connect_engine().await.map_err(engine_error_message)?;
  docker
    .stop_container(&container_id, None::<StopContainerOptions>)
    .await
    .map_err(|err| {
      error!("stop_container failed: {err:?}");
      "Failed to stop container.".to_string()
    })
}

#[tauri::command]
pub async fn restart_container(container_id: String) -> Result<(), String> {
  let docker = connect_engine().await.map_err(engine_error_message)?;
  docker
    .restart_container(&container_id, None::<RestartContainerOptions>)
    .await
    .map_err(|err| {
      error!("restart_container failed: {err:?}");
      "Failed to restart container.".to_string()
    })
}

async fn connect_engine() -> Result<Docker> {
  if let Ok(host) = env::var("DOCKER_HOST") {
    if !host.trim().is_empty() {
      info!("Connecting to engine via DOCKER_HOST");
      return connect_with_host(&host)
        .await
        .with_context(|| format!("DOCKER_HOST connection failed: {host}"));
    }
  }

  let mut last_error: Option<anyhow::Error> = None;
  for socket in DEFAULT_SOCKET_CANDIDATES {
    let expanded = expand_socket_path(socket);
    info!("Connecting to engine via socket: {expanded}");
    match connect_with_host(&expanded).await {
      Ok(docker) => return Ok(docker),
      Err(err) => {
        warn!("Socket candidate failed: {expanded} ({err:?})");
        last_error = Some(err);
      }
    }
  }

  Err(last_error.unwrap_or_else(|| anyhow!("No socket candidates available")))
}

async fn connect_with_host(host: &str) -> Result<Docker> {
  let docker = if host.starts_with("unix://") {
    Docker::connect_with_unix(host, 120, API_DEFAULT_VERSION)?
  } else {
    let resolved = host.replace("tcp://", "http://");
    Docker::connect_with_http(&resolved, 120, API_DEFAULT_VERSION)?
  };

  docker.ping().await.context("engine ping failed")?;
  Ok(docker)
}

fn expand_socket_path(host: &str) -> String {
  let mut expanded = host.to_string();
  if expanded.contains("$XDG_RUNTIME_DIR") {
    if let Ok(dir) = env::var("XDG_RUNTIME_DIR") {
      expanded = expanded.replace("$XDG_RUNTIME_DIR", &dir);
    }
  }
  if expanded.contains("$HOME") {
    if let Ok(home) = env::var("HOME") {
      expanded = expanded.replace("$HOME", &home);
    }
  }
  expanded
}

fn engine_error_message(err: impl std::fmt::Debug) -> String {
  let message = format!("{err:?}").to_lowercase();
  if message.contains("permission") || message.contains("eacces") {
    "Engine access denied. Check socket permissions.".to_string()
  } else if message.contains("timeout") {
    "Engine did not respond. Check it is running.".to_string()
  } else if message.contains("connect") || message.contains("no such file") {
    "Unable to connect to engine. Ensure Docker/Podman is running.".to_string()
  } else {
    "Engine connection failed. Check logs for details.".to_string()
  }
}
