#![allow(warnings)]
use crate::types::*;
use crate::worker::Worker;
use chrono::Utc;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::RwLock;
use tokio::task::JoinHandle;
use tracing::{debug, error, info};
use warp::Filter;

/// Task orchestrator that manages multiple workers with round-robin distribution
pub struct TaskOrchestrator {
    config: OrchestratorConfig,
    workers: Vec<Arc<Worker>>,
    current_worker: AtomicUsize,
    running: AtomicBool,
    start_time: Instant,
    worker_handles: Arc<RwLock<Vec<JoinHandle<()>>>>,
    server_handle: Arc<RwLock<Option<JoinHandle<()>>>>,
}

impl TaskOrchestrator {
    /// Create a new task orchestrator
    pub fn new(config: OrchestratorConfig) -> Result<Self, SystemError> {
        config.validate()?;
        
        info!(
            "Creating orchestrator with {} workers, {} threads each",
            config.num_workers, config.threads_per_worker
        );

        // Create workers
        let mut workers = Vec::new();
        for i in 0..config.num_workers {
            let worker = Arc::new(Worker::new(i, config.threads_per_worker)); // Remove port parameter
            workers.push(worker);
}

        Ok(Self {
            config,
            workers,
            current_worker: AtomicUsize::new(0),
            running: AtomicBool::new(false),
            start_time: Instant::now(),
            worker_handles: Arc::new(RwLock::new(Vec::new())),
            server_handle: Arc::new(RwLock::new(None)),
        })
    }

    /// Start the orchestrator and all workers
    pub async fn start(&self) -> Result<(), SystemError> {
        if self.running.load(Ordering::Acquire) {
            return Err(SystemError::Orchestrator {
                message: "Orchestrator already running".to_string(),
            });
        }

        info!("Starting task orchestrator...");
        self.running.store(true, Ordering::Release);

        // Start all workers
        let mut handles = Vec::new();
        for worker in &self.workers {
            let worker_clone = Arc::clone(worker);
            let handle = tokio::spawn(async move {
                if let Err(e) = worker_clone.start().await {
                    error!("Worker {} failed to start: {}", worker_clone.id, e);
                }
            });
            handles.push(handle);
        }

        // Store worker handles
        {
            let mut worker_handles = self.worker_handles.write().await;
            *worker_handles = handles;
        }

        // Start orchestrator HTTP server
        let server_handle = self.start_http_server().await?;
        {
            let mut server_handle_guard = self.server_handle.write().await;
            *server_handle_guard = Some(server_handle);
        }

        info!(
            "Task orchestrator started on port {} with {} workers",
            self.config.orchestrator_port, self.config.num_workers
        );

        Ok(())
    }

    /// Stop the orchestrator and all workers
    pub async fn stop(&self) {
        info!("Stopping task orchestrator...");
        self.running.store(false, Ordering::Release);

        // Stop all workers
        for worker in &self.workers {
            worker.stop().await;
        }

        // Wait for worker handles to complete
        {
            let mut worker_handles = self.worker_handles.write().await;
            for handle in worker_handles.drain(..) {
                handle.abort();
            }
        }

        // Stop server
        {
            let mut server_handle = self.server_handle.write().await;
            if let Some(handle) = server_handle.take() {
                handle.abort();
            }
        }

        info!("Task orchestrator stopped");
    }

    /// Create a new task and distribute to worker
    pub async fn create_task(&self, request: CreateTaskRequest) -> Result<String, SystemError> {
        if !self.running.load(Ordering::Acquire) {
            return Err(SystemError::Orchestrator {
                message: "Orchestrator not running".to_string(),
            });
        }

        // Convert request to task and validate
        let task = request.into_task()?;
        let task_id = task.id.clone();

        info!(
            "Creating task {} with priority {} for operation {} on input {}",
            task_id, task.priority, task.data.operation, task.data.input
        );

        // Select worker using round-robin
        let worker_index = self.select_worker();
        let worker = &self.workers[worker_index];

        // Add task to selected worker
        worker.add_task(task).await.map_err(|e| SystemError::Task(e))?;

        debug!(
            "Task {} distributed to worker {}",
            task_id, worker_index
        );

        Ok(task_id)
    }

    /// Get task information from any worker
    pub async fn get_task(&self, task_id: &str) -> Result<Task, TaskError> {
        // Search all workers for the task
        for worker in &self.workers {
            if let Some(task) = worker.get_task(task_id) {
                return Ok(task);
            }
        }

        Err(TaskError::TaskNotFound {
            id: task_id.to_string(),
        })
    }

    /// Complete a task on any worker
    pub async fn complete_task(&self, task_id: &str) -> Result<TaskCompletionResponse, TaskError> {
        // Try to complete task on all workers
        for worker in &self.workers {
            if let Ok(true) = worker.complete_task(task_id) {
                return Ok(TaskCompletionResponse {
                    id: task_id.to_string(),
                    status: TaskStatus::Completed,
                    message: "Task completed successfully".to_string(),
                });
            }
        }

        Err(TaskError::TaskNotFound {
            id: task_id.to_string(),
        })
    }

    /// Get system statistics
    pub async fn get_system_stats(&self) -> SystemStats {
        let mut total_processed = 0;
        let mut total_completed = 0;
        let mut total_failed = 0;
        let mut worker_stats = Vec::new();

        // Collect stats from all workers
        for worker in &self.workers {
            let stats = worker.get_stats().await;
            total_processed += stats.tasks_processed;
            total_completed += stats.tasks_completed;
            total_failed += stats.tasks_failed;
            worker_stats.push(stats);
        }

        SystemStats {
            total_tasks_processed: total_processed,
            total_tasks_completed: total_completed,
            total_tasks_failed: total_failed,
            total_workers: self.config.num_workers,
            uptime_seconds: self.start_time.elapsed().as_secs(),
            workers: worker_stats,
        }
    }

    /// Select next worker using round-robin
    fn select_worker(&self) -> usize {
        let current = self.current_worker.fetch_add(1, Ordering::Relaxed);
        current % self.workers.len()
    }

    /// Start the orchestrator HTTP server
    async fn start_http_server(&self) -> Result<JoinHandle<()>, SystemError> {
        let port = self.config.orchestrator_port;
        
        // Clone what we need for the server
        let workers = self.workers.clone();
        
        // Create task endpoint
        let create_task = warp::path!("task" / "create")
            .and(warp::post())
            .and(warp::body::json())
            .and(warp::any().map(move || workers.clone()))
            .and_then(|request: CreateTaskRequest, workers: Vec<Arc<Worker>>| async move {
                // Simple round-robin selection
                static COUNTER: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
                let worker_idx = COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed) % workers.len();
                let worker = &workers[worker_idx];
                
                match request.into_task() {
                    Ok(task) => {
                        let task_id = task.id.clone();
                        match worker.add_task(task).await {
                            Ok(()) => Ok::<_, warp::Rejection>(warp::reply::json(&serde_json::json!({
                                "id": task_id,
                                "status": "pending",
                                "message": "Task created successfully"
                            }))),
                            Err(e) => Ok::<_, warp::Rejection>(warp::reply::json(&serde_json::json!({
                                "error": e.to_string()
                            })))
                        }
                    },
                    Err(e) => Ok::<_, warp::Rejection>(warp::reply::json(&serde_json::json!({
                        "error": e.to_string()
                    })))
                }
            });
    
        // Get task endpoint
        let workers_for_get = self.workers.clone();
        let get_task = warp::path!("task" / String)
            .and(warp::get())
            .and(warp::any().map(move || workers_for_get.clone()))
            .and_then(|task_id: String, workers: Vec<Arc<Worker>>| async move {
                for worker in &workers {
                    if let Some(task) = worker.get_task(&task_id) {
                        return Ok(warp::reply::json(&task));
                    }
                }
                Err(warp::reject::not_found())
            });
    
        // Complete task endpoint
        let workers_for_complete = self.workers.clone();
        let complete_task = warp::path!("task" / String / "complete")
            .and(warp::post())
            .and(warp::any().map(move || workers_for_complete.clone()))
            .and_then(|task_id: String, workers: Vec<Arc<Worker>>| async move {
                for worker in &workers {
                    if let Ok(true) = worker.complete_task(&task_id) {
                        return Ok(warp::reply::json(&TaskCompletionResponse {
                            id: task_id,
                            status: TaskStatus::Completed,
                            message: "Task completed successfully".to_string(),
                        }));
                    }
                }
                Err(warp::reject::not_found())
            });
    
        // Stats endpoint
        let workers_for_stats = self.workers.clone();
        let start_time = self.start_time;
        let config_workers = self.config.num_workers;
        let get_stats = warp::path("stats")
            .and(warp::get())
            .and(warp::any().map(move || (workers_for_stats.clone(), start_time, config_workers)))
            .and_then(|(workers, start_time, num_workers): (Vec<Arc<Worker>>, Instant, usize)| async move {
                let mut total_processed = 0;
                let mut total_completed = 0;
                let mut total_failed = 0;
                let mut worker_stats = Vec::new();
    
                for worker in &workers {
                    let stats = worker.get_stats().await;
                    total_processed += stats.tasks_processed;
                    total_completed += stats.tasks_completed;
                    total_failed += stats.tasks_failed;
                    worker_stats.push(stats);
                }
    
                let system_stats = SystemStats {
                    total_tasks_processed: total_processed,
                    total_tasks_completed: total_completed,
                    total_tasks_failed: total_failed,
                    total_workers: num_workers,
                    uptime_seconds: start_time.elapsed().as_secs(),
                    workers: worker_stats,
                };
    
                Ok::<_, warp::Rejection>(warp::reply::json(&system_stats))
            });
    
        // Health check endpoint
        let health = warp::path("health")
            .and(warp::get())
            .map(|| {
                warp::reply::json(&serde_json::json!({
                    "status": "healthy",
                    "timestamp": Utc::now()
                }))
            });
    
        let routes = create_task
            .or(get_task)
            .or(complete_task)
            .or(get_stats)
            .or(health)
            .with(warp::cors().allow_any_origin())
            .with(warp::log("orchestrator"));
    
        let server = warp::serve(routes).run(([127, 0, 0, 1], port));
    
        let handle = tokio::spawn(async move {
            info!("Orchestrator HTTP server started on port {}", port);
            server.await;
        });
    
        Ok(handle)
    }

}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::*;

    fn create_test_config() -> OrchestratorConfig {
        OrchestratorConfig {
            num_workers: 2,
            threads_per_worker: 2,
            orchestrator_port: 9999,
        }
    }

    #[tokio::test]
    async fn test_orchestrator_creation() {
        let config = create_test_config();
        let orchestrator = TaskOrchestrator::new(config.clone());
        
        assert!(orchestrator.is_ok());
        let orchestrator = orchestrator.unwrap();
        assert_eq!(orchestrator.workers.len(), config.num_workers);
    }

    #[tokio::test]
    async fn test_invalid_config() {
        let mut config = create_test_config();
        config.num_workers = 0; // Invalid
        
        let result = TaskOrchestrator::new(config);
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_round_robin_selection() {
        let config = create_test_config();
        let orchestrator = TaskOrchestrator::new(config).unwrap();
        
        // Test round-robin selection
        assert_eq!(orchestrator.select_worker(), 0);
        assert_eq!(orchestrator.select_worker(), 1);
        assert_eq!(orchestrator.select_worker(), 0); // Wraps around
    }

    #[tokio::test]
    async fn test_create_task_request() {
        let request = CreateTaskRequest {
            id: "test-123".to_string(),
            title: "Test Task".to_string(),
            priority: TaskPriority::High,
            data: TaskData::new(10, Operation::Factorial),
        };

        let task = request.into_task();
        assert!(task.is_ok());
        
        let task = task.unwrap();
        assert_eq!(task.id, "test-123");
        assert_eq!(task.priority, TaskPriority::High);
        assert_eq!(task.status, TaskStatus::Pending);
    }

    #[tokio::test]
    async fn test_system_stats_calculation() {
        let config = create_test_config();
        let orchestrator = TaskOrchestrator::new(config).unwrap();
        
        let stats = orchestrator.get_system_stats().await;
        assert_eq!(stats.total_workers, 2);
        assert_eq!(stats.workers.len(), 2);
        assert_eq!(stats.total_tasks_processed, 0);
    }
}