//! # Task Processing System
//!
//! A high-performance, task processing system with REST API
//! for mathematical calculations.
//!
//! ## Features
//!
//! - **Mathematical Operations**: Supports factorial, fibonacci, and prime_check calculations  
//! - **Multi-threaded Workers**: Configurable number of workers and threads per worker
//! - **Round-Robin Load Balancing**: Distributes tasks across workers
//! - **REST API**: Complete HTTP API for task management
//!
//! ## Quick Start
//!
//! ```rust,no_run
//! use task_processing_system_rs::{TaskOrchestrator, OrchestratorConfig};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let config = OrchestratorConfig::default();
//!     let orchestrator = TaskOrchestrator::new(config)?;
//!     
//!     orchestrator.start().await?;
//!     Ok(())
//! }
//! ```
//!
//! ## Architecture
//!
//! The system consists of several key components:
//!
//! - **Orchestrator**: Coordinates task distribution across workers
//! - **Workers**: Process tasks
//! - **Calculator**: Performs mathematical operations
//! - **Types**: Core data structures and error types

#![allow(warnings)]
pub mod calculations;
pub mod orchestrator;
pub mod types;
pub mod worker;

// Re-export main types for convenience
pub use calculations::Calculator;
pub use orchestrator::TaskOrchestrator;
pub use types::*;
pub use worker::Worker;

/// Result type alias for system operations
pub type SystemResult<T> = Result<T, SystemError>;

/// Task result type alias
pub type TaskResult<T> = Result<T, TaskError>;

/// Validation result type alias  
pub type ValidationResult<T> = Result<T, ValidationError>;

#[cfg(test)]
mod integration_tests {
    use super::*;
    use tokio::time::{sleep, Duration};

    fn create_test_config() -> OrchestratorConfig {
        OrchestratorConfig {
            num_workers: 2,
            threads_per_worker: 2,
            orchestrator_port: 19999,
        }
    }

    #[tokio::test]
    async fn test_system_integration() {
        let config = create_test_config();
        let orchestrator = TaskOrchestrator::new(config).unwrap();

        // Start orchestrator
        let orchestrator_handle = tokio::spawn(async move {
            orchestrator.start().await
        });

        // Give system time to start
        sleep(Duration::from_millis(100)).await;

        // Test creating a task
        let request = CreateTaskRequest {
            id: "integration-test-001".to_string(),
            title: "Integration Test".to_string(),
            priority: TaskPriority::High,
            data: TaskData::new(5, Operation::Factorial),
        };
        orchestrator_handle.abort();
    }

    #[test]
    fn test_calculation_correctness() {
        // Test all supported operations
        assert_eq!(
            Calculator::calculate(Operation::Factorial, 5).unwrap(),
            "120"
        );
        assert_eq!(
            Calculator::calculate(Operation::Fibonacci, 10).unwrap(), 
            "55"
        );
        assert_eq!(
            Calculator::calculate(Operation::PrimeCheck, 17).unwrap(),
            "true"
        );
    }

    #[test]
    fn test_task_lifecycle() {
        let mut task = Task::new(
            "Lifecycle test".to_string(),
            TaskPriority::Medium,
            TaskData::new(10, Operation::Factorial),
        );

        // Initial state
        assert_eq!(task.status, TaskStatus::Pending);
        assert!(task.result.is_none());

        // Process task (simulate worker processing)
        task.set_processing("3628800".to_string());
        assert_eq!(task.status, TaskStatus::Processing);
        assert_eq!(task.result, Some("3628800".to_string()));

        // Complete task (simulate API call)
        let result = task.set_completed();
        assert!(result.is_ok());
        assert_eq!(task.status, TaskStatus::Completed);
        assert!(task.completed_at.is_some());
    }

    #[test]
    fn test_priority_ordering() {
        let high = TaskPriority::High;
        let medium = TaskPriority::Medium;
        let low = TaskPriority::Low;

        assert!(high > medium);
        assert!(medium > low);
        assert!(high > low);
    }

    #[test]
    fn test_error_handling() {
        // Test validation errors
        let invalid_task_data = TaskData {
            task_type: "invalid".to_string(),
            input: 10,
            operation: Operation::Factorial,
        };
        
        assert!(invalid_task_data.validate().is_err());

        // Test calculation errors
        let result = Calculator::calculate(Operation::Factorial, 25);
        assert!(result.is_err());
    }

    #[test]
    fn test_configuration_validation() {
        // Valid config
        let valid_config = OrchestratorConfig {
            num_workers: 3,
            threads_per_worker: 4,
            orchestrator_port: 7000,
        };
        assert!(valid_config.validate().is_ok());

        // Invalid worker count
        let invalid_config = OrchestratorConfig {
            num_workers: 0,
            ..valid_config
        };
        assert!(invalid_config.validate().is_err());
    }
}