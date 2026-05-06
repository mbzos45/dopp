use bollard::query_parameters::{
    ListContainersOptionsBuilder, RestartContainerOptionsBuilder, StartContainerOptions,
    StopContainerOptionsBuilder,
};
use bollard::Docker;
use bollard::config::ContainerSummary;

pub struct DockerClient {
    docker: Option<Docker>,
    runtime: tokio::runtime::Runtime,
}

impl DockerClient {
    pub fn new() -> Self {
        let runtime = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .expect("failed to build tokio runtime");
        let docker = Docker::connect_with_local_defaults().ok();
        Self { docker, runtime }
    }

    pub fn list_containers(&self) -> Result<Vec<ContainerSummary>, String> {
        let docker = self
            .docker
            .as_ref()
            .ok_or_else(|| "Docker connection is unavailable".to_string())?;
        self.runtime
            .block_on(list_containers(docker))
            .map_err(|err| err.to_string())
    }

    pub fn start_container(&self, id: &str) -> Result<(), String> {
        let docker = self
            .docker
            .as_ref()
            .ok_or_else(|| "Docker connection is unavailable".to_string())?;
        self.runtime
            .block_on(docker.start_container(id, None::<StartContainerOptions>))
            .map_err(|err| err.to_string())
    }

    pub fn stop_container(&self, id: &str) -> Result<(), String> {
        let docker = self
            .docker
            .as_ref()
            .ok_or_else(|| "Docker connection is unavailable".to_string())?;
        let options = StopContainerOptionsBuilder::default().t(10).build();
        self.runtime
            .block_on(docker.stop_container(id, Some(options)))
            .map_err(|err| err.to_string())
    }

    pub fn restart_container(&self, id: &str) -> Result<(), String> {
        let docker = self
            .docker
            .as_ref()
            .ok_or_else(|| "Docker connection is unavailable".to_string())?;
        let options = RestartContainerOptionsBuilder::default().t(10).build();
        self.runtime
            .block_on(docker.restart_container(id, Some(options)))
            .map_err(|err| err.to_string())
    }
}

async fn list_containers(docker: &Docker) -> Result<Vec<ContainerSummary>, bollard::errors::Error> {
    let options = ListContainersOptionsBuilder::default().all(true).build();
    Ok(docker.list_containers(Some(options)).await?)
}
