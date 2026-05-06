use anyhow::{Result};
use bollard::Docker;
use bollard::config::ContainerSummary;
use bollard::query_parameters::{
    ListContainersOptionsBuilder, RestartContainerOptionsBuilder, StartContainerOptions,
    StopContainerOptionsBuilder,
};
use crossbeam_channel::Sender;
use egui::Context as EguiContext;

pub struct DockerRunner {
    docker: Option<Docker>,
    runtime: tokio::runtime::Runtime,
    tx: Sender<UiEvent>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ContainerActionKind {
    Start,
    Stop,
    Restart,
}

impl ContainerActionKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Start => "start",
            Self::Stop => "stop",
            Self::Restart => "restart",
        }
    }
}

pub enum UiEvent {
    ContainersRefreshed(Result<Vec<ContainerSummary>, String>),
    ContainerActionFinished {
        id: String,
        action: ContainerActionKind,
        result: Result<(), String>,
    },
}

impl DockerRunner {
    pub fn new(tx: Sender<UiEvent>) -> Self {
        let runtime = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .expect("failed to build tokio runtime");
        let docker = Docker::connect_with_local_defaults().ok();
        Self { docker, runtime, tx }
    }

    pub fn spawn_refresh(&self, ctx: EguiContext) {
        let docker = self.docker.clone();
        let tx = self.tx.clone();
        self.runtime.spawn(async move {
            let result = match docker {
                Some(docker) => list_containers(&docker).await.map_err(|err| err.to_string()),
                None => Err("Docker connection is unavailable".to_string()),
            };
            let _ = tx.send(UiEvent::ContainersRefreshed(result));
            ctx.request_repaint();
        });
    }

    pub fn spawn_action(&self, id: String, action: ContainerActionKind, ctx: EguiContext) {
        let docker = self.docker.clone();
        let tx = self.tx.clone();
        self.runtime.spawn(async move {
            let (action_result, refresh_result) = match docker {
                Some(docker) => {
                    let action_result =
                        run_action(&docker, &id, action).await.map_err(|err| err.to_string());
                    let refresh_result =
                        list_containers(&docker).await.map_err(|err| err.to_string());
                    (action_result, refresh_result)
                }
                None => {
                    let err = "Docker connection is unavailable".to_string();
                    (Err(err.clone()), Err(err))
                }
            };

            let _ = tx.send(UiEvent::ContainerActionFinished {
                id: id.clone(),
                action,
                result: action_result,
            });
            let _ = tx.send(UiEvent::ContainersRefreshed(refresh_result));
            ctx.request_repaint();
        });
    }
}

async fn list_containers(docker: &Docker) -> Result<Vec<ContainerSummary>> {
    let options = ListContainersOptionsBuilder::default().all(true).build();
    Ok(docker.list_containers(Some(options)).await?)
}

async fn run_action(docker: &Docker, id: &str, action: ContainerActionKind) -> Result<()> {
    match action {
        ContainerActionKind::Start => {
            docker.start_container(id, None::<StartContainerOptions>).await?;
            Ok(())
        }
        ContainerActionKind::Stop => {
            let options = StopContainerOptionsBuilder::default().t(10).build();
            docker.stop_container(id, Some(options)).await?;
            Ok(())
        }
        ContainerActionKind::Restart => {
            let options = RestartContainerOptionsBuilder::default().t(10).build();
            docker.restart_container(id, Some(options)).await?;
            Ok(())
        }
    }
}
