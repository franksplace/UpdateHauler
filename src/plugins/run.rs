use async_trait::async_trait;

use super::{Plugin, PluginAction, PluginActionType, PluginMetadata};
use crate::config::Config;
use crate::insights::Insights;
use crate::logger::Logger;
use anyhow::Result;

pub struct RunPlugin;

#[async_trait]
impl Plugin for RunPlugin {
    fn name(&self) -> &str {
        "run"
    }

    fn get_metadata(&self) -> PluginMetadata {
        PluginMetadata {
            name: "run".to_string(),
            description: "Run an arbitrary command".to_string(),
            actions: vec![PluginAction {
                name: "run".to_string(),
                description: "Run a command specified via --cmd".to_string(),
                action_type: Some(PluginActionType::Update),
            }],
        }
    }

    async fn check_available(&self, _config: &Config, _insights: &Insights) -> bool {
        true
    }

    async fn update(
        &self,
        config: &Config,
        _insights: &Insights,
        logger: &mut Logger,
    ) -> Result<()> {
        if config.cmd_args.is_empty() {
            logger.log("No command specified. Use: updatehauler run --cmd <command>");
            return Ok(());
        }

        let (program, args) = if config.cmd_args.len() == 1 {
            let parts: Vec<&str> = config.cmd_args[0].split_whitespace().collect();
            if parts.is_empty() {
                logger.log("No command specified. Use: updatehauler run --cmd <command>");
                return Ok(());
            }
            let args: Vec<&str> = parts[1..].to_vec();
            (parts[0].to_string(), args)
        } else {
            let (program, rest) = config.cmd_args.split_first().unwrap();
            let args: Vec<&str> = rest.iter().map(|s| s.as_str()).collect();
            (program.clone(), args)
        };

        super::run_cmd(config, logger, true, &program, &args)?;
        Ok(())
    }

    async fn save(
        &self,
        _config: &Config,
        _insights: &Insights,
        _logger: &mut Logger,
    ) -> Result<()> {
        Ok(())
    }

    async fn restore(
        &self,
        _config: &Config,
        _insights: &Insights,
        _logger: &mut Logger,
    ) -> Result<()> {
        Ok(())
    }
}
