use crate::context::FmtContext;
use anyhow::{anyhow, Result};
use colored::Colorize;
use serde::Deserialize;
use std::sync::{mpsc, Arc};
use std::thread;
use std::{fs, path::Path};

#[derive(Clone, Debug, Deserialize)]
pub struct Config {
    #[serde(default = "default_max_width")]
    pub max_width: usize,

    #[serde(default = "default_indent_size")]
    pub indent_size: usize,
}

fn default_max_width() -> usize {
    80
}

fn default_indent_size() -> usize {
    2
}

impl Default for Config {
    fn default() -> Self {
        Self {
            max_width: default_max_width(),
            indent_size: default_indent_size(),
        }
    }
}

impl Config {
    pub fn new(max_width: usize) -> Self {
        Self {
            max_width,
            indent_size: 2,
        }
    }

    pub fn from_file(path: &str) -> Result<Self> {
        let content =
            fs::read_to_string(path).map_err(|e| anyhow!("Failed to read config file: {}", e))?;
        let config: Config =
            toml::from_str(&content).map_err(|e| anyhow!("Failed to parse config file: {}", e))?;
        Ok(config)
    }

    pub fn max_width(&self) -> usize {
        self.max_width
    }

    pub fn indent_size(&self) -> usize {
        self.indent_size
    }
}

#[derive(Clone, Debug)]
pub struct Session {
    config: Config,
    source_files: Vec<String>,
    //pub errors: ReportedErrors,
}

impl Session {
    pub fn new(config: Config, source_files: Vec<String>) -> Self {
        Self {
            config,
            source_files,
            //errors: ReportedErrors::default(),
        }
    }

    pub fn config(&self) -> &Config {
        &self.config
    }

    pub fn create_session_from_config(
        config_path: Option<&str>,
        source_files: Vec<String>,
    ) -> Result<Session> {
        let config = match config_path {
            Some(path) => Config::from_file(path)
                .map_err(|e| anyhow!(format!("{}: {}", e.to_string().yellow(), path)))?,
            None => Config::default(),
        };
        Ok(Session::new(config, source_files))
    }

    pub fn format(&self) {
        let file = &self.source_files[0];
        let source_code = fs::read_to_string(Path::new(file))
            .map_err(|e| {
                anyhow!(format!(
                    "Failed to read file: {} {}",
                    &file.red(),
                    e.to_string().yellow()
                ))
            })
            .unwrap();

        let context = FmtContext::new(&self.config, source_code);
        context.format_one_file();
    }
}
