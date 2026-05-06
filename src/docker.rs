use anyhow::{Context, Result};
use bollard::Docker;
use bollard::config::ContainerSummary;
use bollard::query_parameters::{
    ListContainersOptionsBuilder, RestartContainerOptionsBuilder, StartContainerOptions,
    StopContainerOptionsBuilder,
};

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

    pub fn list_containers(&self) -> Result<Vec<ContainerSummary>> {
        let docker = self
            .docker
            .as_ref()
            .context("Docker connection is unavailable")?;
        self.runtime.block_on(list_containers(docker))
    }

    pub fn start_container(&self, id: &str) -> Result<()> {
        let docker = self
            .docker
            .as_ref()
            .context("Docker connection is unavailable")?;
        self.runtime
            .block_on(docker.start_container(id, None::<StartContainerOptions>))?;
        Ok(())
    }

    pub fn stop_container(&self, id: &str) -> Result<()> {
        let docker = self
            .docker
            .as_ref()
            .context("Docker connection is unavailable")?;
        let options = StopContainerOptionsBuilder::default().t(10).build();
        self.runtime
            .block_on(docker.stop_container(id, Some(options)))?;
        Ok(())
    }

    pub fn restart_container(&self, id: &str) -> Result<()> {
        let docker = self
            .docker
            .as_ref()
            .context("Docker connection is unavailable")?;
        let options = RestartContainerOptionsBuilder::default().t(10).build();
        self.runtime
            .block_on(docker.restart_container(id, Some(options)))?;
        Ok(())
    }
}

async fn list_containers(docker: &Docker) -> Result<Vec<ContainerSummary>> {
    let options = ListContainersOptionsBuilder::default().all(true).build();
    Ok(docker.list_containers(Some(options)).await?)
}
