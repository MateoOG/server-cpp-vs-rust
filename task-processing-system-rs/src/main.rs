mod calculations;
mod orchestrator;
mod types;
mod worker;

use crate::orchestrator::TaskOrchestrator;
use crate::types::{OrchestratorConfig};
use clap::{Arg, ArgMatches, Command};
use config::{Config, ConfigError, File};
use serde::Deserialize;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::signal;
use tracing::{error, info, warn};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

/// Application configuration that can be loaded from file or CLI args
#[derive(Debug, Deserialize)]
struct AppConfig {
    #[serde(flatten)]
    orchestrator: OrchestratorConfig,
    
    #[serde(default = "default_log_level")]
    log_level: String,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            orchestrator: OrchestratorConfig::default(),
            log_level: default_log_level(),
        }
    }
}

fn default_log_level() -> String {
    "info".to_string()
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Setup command line interface
    let matches = create_cli().get_matches();

    // Load configuration
    let config = load_configuration(&matches)?;

    // Initialize logging
    setup_logging(&config.log_level)?;

    // Print system information
    print_system_info(&config);

    // Create and start orchestrator
    let orchestrator_config = config.orchestrator.clone();
    let orchestrator = Arc::new(TaskOrchestrator::new(orchestrator_config)?);

    // Setup graceful shutdown
    let shutdown_orchestrator = Arc::clone(&orchestrator);
    tokio::spawn(async move {
        wait_for_shutdown_signal().await;
        info!("Shutdown signal received, stopping orchestrator...");
        shutdown_orchestrator.stop().await;
    });

    // Start the system
    info!("Starting Task Processing System...");
    match orchestrator.start().await {
        Ok(()) => {
            info!("Task Processing System started successfully!");
            print_usage_instructions(&config);
            
            // Keep the main thread alive
            tokio::signal::ctrl_c().await?;
            info!("Task Processing System shutdown complete");
        }
        Err(e) => {
            error!("Failed to start Task Processing System: {}", e);
            std::process::exit(1);
        }
    }

    Ok(())
}

/// Create command line interface
fn create_cli() -> Command {
    Command::new("task-processor")
        .about("High-performance task processing system")
        .version(env!("CARGO_PKG_VERSION"))
        .arg(
            Arg::new("workers")
                .short('w')
                .long("workers")
                .value_name("NUM")
                .help("Number of worker nodes (default: 3, max: 50)")
                .value_parser(clap::value_parser!(usize))
        )
        .arg(
            Arg::new("threads")
                .short('t')
                .long("threads")
                .value_name("NUM")
                .help("Threads per worker (default: 4, max: 32)")
                .value_parser(clap::value_parser!(usize))
        )
        .arg(
            Arg::new("orchestrator-port")
                .short('o')
                .long("orchestrator-port")
                .value_name("NUM")
                .help("Orchestrator port (default: 7000)")
                .value_parser(clap::value_parser!(u16))
        )
        .arg(
            Arg::new("config")
                .short('c')
                .long("config")
                .value_name("FILE")
                .help("Configuration file (JSON/TOML)")
        )
        .arg(
            Arg::new("log-level")
                .short('l')
                .long("log-level")
                .value_name("LEVEL")
                .help("Log level (error, warn, info, debug, trace)")
                .default_value("info")
        )
}

/// Load configuration from file and CLI arguments
fn load_configuration(matches: &ArgMatches) -> Result<AppConfig, ConfigError> {
    let mut config_builder = Config::builder();

    // Load from config file if specified
    if let Some(config_path) = matches.get_one::<String>("config") {
        let path = PathBuf::from(config_path);
        if path.exists() {
            info!("Loading configuration from: {}", config_path);
            config_builder = config_builder.add_source(File::from(path));
        } else {
            warn!("Config file not found: {}", config_path);
        }
    } else {
        // Try to load default config files
        for config_file in &["config.toml", "config.json"] {
            let path = PathBuf::from(config_file);
            if path.exists() {
                info!("Loading default configuration from: {}", config_file);
                config_builder = config_builder.add_source(File::from(path));
                break;
            }
        }
    }

    // Override with command line arguments
    if let Some(&workers) = matches.get_one::<usize>("workers") {
        config_builder = config_builder.set_override("num_workers", workers as i64)?;
    }
    
    if let Some(&threads) = matches.get_one::<usize>("threads") {
        config_builder = config_builder.set_override("threads_per_worker", threads as i64)?;
    }
    
    if let Some(&orchestrator_port) = matches.get_one::<u16>("orchestrator-port") {
        config_builder = config_builder.set_override("orchestrator_port", orchestrator_port as i64)?;
    }

    if let Some(log_level) = matches.get_one::<String>("log-level") {
        config_builder = config_builder.set_override("log_level", log_level.as_str())?;
    }

    let config = config_builder.build()?.try_deserialize::<AppConfig>()?;
    
    // Validate configuration
    config.orchestrator.validate().map_err(|e| {
        ConfigError::Message(format!("Configuration validation failed: {}", e))
    })?;

    Ok(config)
}

/// Setup logging based on configuration
fn setup_logging(log_level: &str) -> Result<(), Box<dyn std::error::Error>> {
    let level = match log_level.to_lowercase().as_str() {
        "error" => tracing::Level::ERROR,
        "warn" => tracing::Level::WARN,
        "info" => tracing::Level::INFO,
        "debug" => tracing::Level::DEBUG,
        "trace" => tracing::Level::TRACE,
        _ => tracing::Level::INFO,
    };

    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| {
                    format!("task_processing_system={},tower_http=debug,warp=info", level).into()
                }),
        )
        .with(tracing_subscriber::fmt::layer().with_target(false))
        .init();

    Ok(())
}

/// Print system information and configuration
fn print_system_info(config: &AppConfig) {
    println!("=== Task Processing System ===");
    println!("Configuration:");
    println!("  Workers: {}", config.orchestrator.num_workers);
    println!("  Threads per worker: {}", config.orchestrator.threads_per_worker);
    println!("  Total processing threads: {}", 
             config.orchestrator.num_workers * config.orchestrator.threads_per_worker);
    println!("  Orchestrator port: {}", config.orchestrator.orchestrator_port);
    println!("  Log level: {}", config.log_level);
    println!();

    println!("Workers: {} internal processing nodes (not externally accessible)", config.orchestrator.num_workers);
    println!();

    println!("Orchestrator Endpoints:");
    println!("  API Base: http://127.0.0.1:{}", config.orchestrator.orchestrator_port);
    println!("  Health: GET /health");
    println!("  Create Task: POST /task/create");
    println!("  Get Task: GET /task/{{id}}");
    println!("  Complete Task: POST /task/{{id}}/complete");
    println!("  Statistics: GET /stats");
    println!();

    print_task_workflow_info();
}

/// Print task workflow information
fn print_task_workflow_info() {
    println!("Task Completion Workflow:");
    println!("  1. Task created -> STATUS: pending");
    println!("  2. Worker processes -> STATUS: processing (calculation done)");
    println!("  3. API call to complete -> STATUS: completed");
    println!("  Tasks can ONLY be marked completed via POST /task/{{id}}/complete");
    println!();

    println!("JSON Format Example:");
    println!(r#"{{
  "id": "task-001",
  "title": "Process calculation",
  "priority": 3,
  "data": {{
    "type": "calculation",
    "input": 10,
    "operation": "factorial"
  }}
}}"#);
    println!();
}

/// Print usage instructions after startup
fn print_usage_instructions(config: &AppConfig) {
    println!("Task Processing System started successfully!");
    println!("Task distribution system is ACTIVE.");
    println!("Tasks must be completed via POST /task/{{id}}/complete API call.");
    println!();
    
    println!("Quick Test Commands:");
    println!("# 1. Create a factorial task");
    println!(r#"curl -X POST http://localhost:{}/task/create \
  -H "Content-Type: application/json" \
  -d '{{
    "id": "example-001",
    "title": "Calculate 10!",
    "priority": 3,
    "data": {{
      "type": "calculation",
      "input": 10,
      "operation": "factorial"
    }}
  }}'"#, config.orchestrator.orchestrator_port);
    println!();
    
    println!("# 2. Check task status");
    println!("curl http://localhost:{}/task/example-001", config.orchestrator.orchestrator_port);
    println!();
    
    println!("# 3. Complete the task (REQUIRED step)");
    println!("curl -X POST http://localhost:{}/task/example-001/complete", config.orchestrator.orchestrator_port);
    println!();
    
    println!("# 4. Check system statistics");
    println!("curl http://localhost:{}/stats", config.orchestrator.orchestrator_port);
    println!();
    
    println!("Press Ctrl+C to stop the system");
}

/// Wait for shutdown signals
async fn wait_for_shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = AppConfig::default();
        assert_eq!(config.orchestrator.num_workers, 3);
        assert_eq!(config.orchestrator.threads_per_worker, 4);
        assert_eq!(config.orchestrator.orchestrator_port, 7000);
        assert_eq!(config.log_level, "info");
    }

    #[test]
    fn test_cli_creation() {
        let cli = create_cli();
        assert_eq!(cli.get_name(), "task-processor");
        
        // Test that all expected arguments are present
        let args: Vec<_> = cli.get_arguments().map(|a| a.get_id()).collect();
        assert!(args.contains(&&clap::Id::from("workers")));
        assert!(args.contains(&&clap::Id::from("threads")));
        assert!(args.contains(&&clap::Id::from("orchestrator-port")));
        assert!(args.contains(&&clap::Id::from("config")));
        assert!(args.contains(&&clap::Id::from("log-level")));
    }
}