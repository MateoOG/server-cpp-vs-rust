#![allow(warnings)]
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fmt;
use uuid::Uuid;

/// Task priority levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(from = "u8", into = "u8")]
pub enum TaskPriority {
    Low = 1,
    Medium = 2,
    High = 3,
}

impl Default for TaskPriority {
    fn default() -> Self {
        TaskPriority::Medium
    }
}

impl fmt::Display for TaskPriority {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", *self as u8)
    }
}

impl From<u8> for TaskPriority {
    fn from(value: u8) -> Self {
        match value {
            1 => TaskPriority::Low,
            2 => TaskPriority::Medium,
            3 => TaskPriority::High,
            _ => TaskPriority::Medium,
        }
    }
}

impl From<TaskPriority> for u8 {
    fn from(priority: TaskPriority) -> Self {
        priority as u8
    }
}


/// Task execution status
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TaskStatus {
    Pending,    // Task created, waiting to be processed
    Processing, // Task calculation completed, awaiting API completion
    Completed,  // Task marked complete via API call
    Failed,     // Task processing failed
}

impl Default for TaskStatus {
    fn default() -> Self {
        TaskStatus::Pending
    }
}

/// Mathematical operations supported by the system
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Operation {
    #[serde(rename = "factorial")]
    Factorial,
    #[serde(rename = "fibonacci")]
    Fibonacci,
    #[serde(rename = "prime_check")]
    PrimeCheck,
}

impl fmt::Display for Operation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Operation::Factorial => "factorial",
            Operation::Fibonacci => "fibonacci",
            Operation::PrimeCheck => "prime_check",
        };
        write!(f, "{}", s)
    }
}

/// Task data payload containing calculation parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskData {
    #[serde(rename = "type")]
    pub task_type: String, // Always "calculation" for our use case
    pub input: u64,
    pub operation: Operation,
}

impl TaskData {
    pub fn new(input: u64, operation: Operation) -> Self {
        Self {
            task_type: "calculation".to_string(),
            input,
            operation,
        }
    }

    /// Validate task data input constraints
    pub fn validate(&self) -> Result<(), ValidationError> {
        if self.task_type != "calculation" {
            return Err(ValidationError::InvalidTaskType(self.task_type.clone()));
        }

        match self.operation {
            Operation::Factorial => {
                if self.input > 20 {
                    return Err(ValidationError::InputTooLarge {
                        operation: self.operation.clone(),
                        input: self.input,
                        max_allowed: 20,
                    });
                }
            }
            Operation::Fibonacci => {
                if self.input > 93 {
                    return Err(ValidationError::InputTooLarge {
                        operation: self.operation.clone(),
                        input: self.input,
                        max_allowed: 93,
                    });
                }
            }
            Operation::PrimeCheck => {
                if self.input > u64::MAX / 2 {
                    return Err(ValidationError::InputTooLarge {
                        operation: self.operation.clone(),
                        input: self.input,
                        max_allowed: u64::MAX / 2,
                    });
                }
            }
        }

        Ok(())
    }
}

/// Main Task structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub id: String,
    pub title: String,
    pub priority: TaskPriority,
    pub created_at: DateTime<Utc>,
    pub data: TaskData,
    pub status: TaskStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_message: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub completed_at: Option<DateTime<Utc>>,
}

impl Task {
    /// Create a new task with generated ID
    pub fn new(title: String, priority: TaskPriority, data: TaskData) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            title,
            priority,
            created_at: Utc::now(),
            data,
            status: TaskStatus::Pending,
            result: None,
            error_message: None,
            completed_at: None,
        }
    }

    /// Create a new task with specific ID (for testing or external systems)
    pub fn with_id(
        id: String,
        title: String,
        priority: TaskPriority,
        data: TaskData,
    ) -> Self {
        Self {
            id,
            title,
            priority,
            created_at: Utc::now(),
            data,
            status: TaskStatus::Pending,
            result: None,
            error_message: None,
            completed_at: None,
        }
    }

    /// Mark task as processing with result
    pub fn set_processing(&mut self, result: String) {
        self.status = TaskStatus::Processing;
        self.result = Some(result);
    }

    /// Mark task as completed (can only be done via API call)
    pub fn set_completed(&mut self) -> Result<(), TaskError> {
        if self.status != TaskStatus::Processing {
            return Err(TaskError::InvalidStatusTransition {
                current: self.status.clone(),
                requested: TaskStatus::Completed,
            });
        }
        self.status = TaskStatus::Completed;
        self.completed_at = Some(Utc::now());
        Ok(())
    }

    /// Mark task as failed
    pub fn set_failed(&mut self, error_message: String) {
        self.status = TaskStatus::Failed;
        self.error_message = Some(error_message);
    }

    /// Get task age in seconds
    pub fn age_seconds(&self) -> i64 {
        (Utc::now() - self.created_at).num_seconds()
    }

    /// Validate task data
    pub fn validate(&self) -> Result<(), ValidationError> {
        if self.id.is_empty() {
            return Err(ValidationError::EmptyTaskId);
        }
        if self.title.is_empty() {
            return Err(ValidationError::EmptyTitle);
        }
        self.data.validate()
    }
}

/// Task creation request from API
#[derive(Debug, Deserialize)]
pub struct CreateTaskRequest {
    #[serde(default = "generate_task_id")]
    pub id: String,
    pub title: String,
    #[serde(default)]
    pub priority: TaskPriority,
    pub data: TaskData,
}

fn generate_task_id() -> String {
    Uuid::new_v4().to_string()
}

impl CreateTaskRequest {
    pub fn into_task(self) -> Result<Task, ValidationError> {
        let task = Task::with_id(self.id, self.title, self.priority, self.data);
        task.validate()?;
        Ok(task)
    }
}

/// Task completion response
#[derive(Debug, Serialize)]
pub struct TaskCompletionResponse {
    pub id: String,
    pub status: TaskStatus,
    pub message: String,
}

/// Worker statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkerStats {
    pub id: usize,
    pub tasks_processed: u64,
    pub tasks_completed: u64,
    pub tasks_failed: u64,
    pub current_load: usize,
    pub uptime_seconds: u64,
    pub is_healthy: bool,
}

/// System-wide statistics
#[derive(Debug, Serialize)]
pub struct SystemStats {
    pub total_tasks_processed: u64,
    pub total_tasks_completed: u64,
    pub total_tasks_failed: u64,
    pub total_workers: usize,
    pub uptime_seconds: u64,
    pub workers: Vec<WorkerStats>,
}

/// Configuration structures
#[derive(Debug, Clone, Deserialize)]
pub struct OrchestratorConfig {
    pub num_workers: usize,
    pub threads_per_worker: usize,
    pub orchestrator_port: u16,
}

impl Default for OrchestratorConfig {
    fn default() -> Self {
        Self {
            num_workers: 3,
            threads_per_worker: 4,
            orchestrator_port: 7000,
        }
    }
}

impl OrchestratorConfig {
    pub fn validate(&self) -> Result<(), ValidationError> {
        if self.num_workers == 0 || self.num_workers > 50 {
            return Err(ValidationError::InvalidWorkerCount(self.num_workers));
        }
        
        if self.threads_per_worker == 0 || self.threads_per_worker > 32 {
            return Err(ValidationError::InvalidThreadCount(self.threads_per_worker));
        }

        if self.orchestrator_port <= 1024 {
            return Err(ValidationError::InvalidPort(self.orchestrator_port));
        }

        Ok(())
    }
}

/// Error types
#[derive(Debug, thiserror::Error)]
pub enum TaskError {
    #[error("Invalid status transition from {current:?} to {requested:?}")]
    InvalidStatusTransition {
        current: TaskStatus,
        requested: TaskStatus,
    },
    
    #[error("Task not found: {id}")]
    TaskNotFound { id: String },
    
    #[error("Task already exists: {id}")]
    TaskAlreadyExists { id: String },
    
    #[error("Calculation error: {message}")]
    CalculationError { message: String },
}

#[derive(Debug, thiserror::Error)]
pub enum ValidationError {
    #[error("Empty task ID")]
    EmptyTaskId,
    
    #[error("Empty title")]
    EmptyTitle,
    
    #[error("Invalid task type: {0}, expected 'calculation'")]
    InvalidTaskType(String),
    
    #[error("Input {input} too large for operation {operation}, max allowed: {max_allowed}")]
    InputTooLarge {
        operation: Operation,
        input: u64,
        max_allowed: u64,
    },
    
    #[error("Invalid worker count: {0}, must be between 1 and 50")]
    InvalidWorkerCount(usize),
    
    #[error("Invalid thread count: {0}, must be between 1 and 32")]
    InvalidThreadCount(usize),
    
    #[error("Invalid port: {0}, must be between 1024 and 65535")]
    InvalidPort(u16),
    
    #[error("Port conflict: orchestrator port {orchestrator_port} conflicts with worker port range {worker_port_range:?}")]
    PortConflict {
        orchestrator_port: u16,
        worker_port_range: (u16, u16),
    },
}

#[derive(Debug, thiserror::Error)]
pub enum SystemError {
    #[error("Worker error: {message}")]
    Worker { message: String },
    
    #[error("Orchestrator error: {message}")]
    Orchestrator { message: String },
    
    #[error("Configuration error: {0}")]
    Config(#[from] ValidationError),
    
    #[error("Task error: {0}")]
    Task(#[from] TaskError),
    
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
}