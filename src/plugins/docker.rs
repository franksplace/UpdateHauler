use async_trait::async_trait;

use super::{Plugin, PluginAction, PluginActionType, PluginMetadata};
use crate::config::Config;
use crate::insights::Insights;
use crate::logger::Logger;
use anyhow::Result;
use duct::cmd;

pub struct DockerPlugin;

impl DockerPlugin {
    fn daemon_running(logger: &mut Logger) -> bool {
        let result = cmd("docker", &["info"])
            .stdout_capture()
            .stderr_capture()
            .run();
        match result {
            Ok(output) => output.status.success(),
            Err(_) => {
                logger.log("Docker daemon is not running, skipping prune");
                false
            }
        }
    }
}

#[async_trait]
impl Plugin for DockerPlugin {
    fn name(&self) -> &str {
        "docker"
    }

    fn get_metadata(&self) -> PluginMetadata {
        PluginMetadata {
            name: "docker".to_string(),
            description: "Clean up unused Docker data (prune)".to_string(),
            actions: vec![PluginAction {
                name: "docker".to_string(),
                description: "Prune unused Docker images, containers, and networks".to_string(),
                action_type: Some(PluginActionType::Update),
            }],
        }
    }

    async fn check_available(&self, _config: &Config, insights: &Insights) -> bool {
        insights.has_docker
    }

    async fn update(
        &self,
        config: &Config,
        _insights: &Insights,
        logger: &mut Logger,
    ) -> Result<()> {
        if !Self::daemon_running(logger) {
            return Ok(());
        }
        super::run_cmd(
            config,
            logger,
            true,
            "docker",
            &["system", "prune", "--force"],
        )?;
        Ok(())
    }
}
