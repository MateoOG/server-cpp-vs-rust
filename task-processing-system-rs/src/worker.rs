#![allow(warnings)]
use crate::calculations::Calculator;
use crate::types::*;
use dashmap::DashMap;
use std::collections::VecDeque;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{Mutex, Notify};
use tokio::task::JoinHandle;
use tracing::{debug, error, info};

/// Worker node that processes tasks
pub struct Worker {
    pub id: usize,
    config: WorkerConfig,
    
    // Task storage and processing
    tasks: Arc<DashMap<String, Task>>,
    task_queue: Arc<Mutex<VecDeque<Task>>>,
    queue_notify: Arc<Notify>,
    
    // Statistics
    tasks_processed: Arc<AtomicU64>,
    tasks_completed: Arc<AtomicU64>,
    tasks_failed: Arc<AtomicU64>,
    start_time: Instant,
    
    // Control
    running: Arc<AtomicBool>,
    shutdown_notify: Arc<Notify>,
}

#[derive(Debug, Clone)]
pub struct WorkerConfig {
    pub id: usize,
    pub num_threads: usize,
}

impl Worker {
    /// Create a new worker instance
    pub fn new(id: usize, num_threads: usize) -> Self {
        Self {
            id,
            config: WorkerConfig {
                id,
                num_threads,
            },
            tasks: Arc::new(DashMap::new()),
            task_queue: Arc::new(Mutex::new(VecDeque::new())),
            queue_notify: Arc::new(Notify::new()),
            tasks_processed: Arc::new(AtomicU64::new(0)),
            tasks_completed: Arc::new(AtomicU64::new(0)),
            tasks_failed: Arc::new(AtomicU64::new(0)),
            start_time: Instant::now(),
            running: Arc::new(AtomicBool::new(false)),
            shutdown_notify: Arc::new(Notify::new()),
        }
    }

    /// Start the worker with processing threads
    pub async fn start(&self) -> Result<(), SystemError> {
        if self.running.load(Ordering::Acquire) {
            return Err(SystemError::Worker {
                message: format!("Worker {} already running", self.id),
            });
        }
    
        info!("Starting worker {} (internal only)", self.id);
        self.running.store(true, Ordering::Release);
    
        // Start processing threads only
        let mut thread_handles = Vec::new();
        for thread_id in 0..self.config.num_threads {
            let handle = self.spawn_processing_thread(thread_id);
            thread_handles.push(handle);
        }
    
        info!("Worker {} started successfully with {} threads", self.id, self.config.num_threads);
    
        // Wait for shutdown signal
        self.shutdown_notify.notified().await;
        
        info!("Shutting down worker {}", self.id);
        self.running.store(false, Ordering::Release);
    
        // Cancel processing threads only
        for handle in thread_handles {
            handle.abort();
        }
    
        Ok(())
    }

    /// Stop the worker gracefully
    pub async fn stop(&self) {
        info!("Stopping worker {}", self.id);
        self.running.store(false, Ordering::Release);
        self.shutdown_notify.notify_waiters();
    }

    /// Add a task to the worker's queue
    pub async fn add_task(&self, task: Task) -> Result<(), TaskError> {
        debug!("Worker {} received task {}", self.id, task.id);
        
        // Validate task before adding
        task.validate().map_err(|e| TaskError::CalculationError {
            message: format!("Task validation failed: {}", e),
        })?;

        // Store task
        if self.tasks.contains_key(&task.id) {
            return Err(TaskError::TaskAlreadyExists { id: task.id });
        }

        let task_id = task.id.clone();
        self.tasks.insert(task_id.clone(), task.clone());

        // Add to task queue
        {
            let mut queue = self.task_queue.lock().await;
            queue.push_back(task);
        }

        // Notify processing threads
        self.queue_notify.notify_one();
        
        debug!("Task {} added to worker {} queue", task_id, self.id);
        Ok(())
    }

    /// Get task information
    pub fn get_task(&self, task_id: &str) -> Option<Task> {
        self.tasks.get(task_id).map(|entry| entry.clone())
    }

    /// Complete a task (can only be done via API call)
    pub fn complete_task(&self, task_id: &str) -> Result<bool, TaskError> {
        if let Some(mut task_entry) = self.tasks.get_mut(task_id) {
            let result = task_entry.set_completed();
            match result {
                Ok(()) => {
                    self.tasks_completed.fetch_add(1, Ordering::Relaxed);
                    info!("Task {} completed on worker {}", task_id, self.id);
                    Ok(true)
                }
                Err(e) => Err(e),
            }
        } else {
            Ok(false) // Task not found on this worker
        }
    }

    /// Get worker statistics
    pub async fn get_stats(&self) -> WorkerStats {
        let current_queue_size = {
            let queue = self.task_queue.lock().await;
            queue.len()
        };
        
        WorkerStats {
            id: self.id,
            tasks_processed: self.tasks_processed.load(Ordering::Relaxed),
            tasks_completed: self.tasks_completed.load(Ordering::Relaxed),
            tasks_failed: self.tasks_failed.load(Ordering::Relaxed),
            current_load: current_queue_size,
            uptime_seconds: self.start_time.elapsed().as_secs(),
            is_healthy: self.running.load(Ordering::Acquire),
        }
    }

    /// Spawn a processing thread
    fn spawn_processing_thread(&self, thread_id: usize) -> JoinHandle<()> {
        let worker_id = self.id;
        let tasks = Arc::clone(&self.tasks);
        let task_queue = Arc::clone(&self.task_queue);
        let queue_notify = Arc::clone(&self.queue_notify);
        let running = Arc::clone(&self.running);
        let tasks_processed = Arc::clone(&self.tasks_processed);
        let tasks_failed = Arc::clone(&self.tasks_failed);

        tokio::spawn(async move {
            info!("Processing thread {} started for worker {}", thread_id, worker_id);

            while running.load(Ordering::Acquire) {
                // Wait for tasks or shutdown signal
                tokio::select! {
                    _ = queue_notify.notified() => {
                        // Process available tasks
                        while let Some(task) = {
                            let mut queue = task_queue.lock().await;
                            queue.pop_front()
                        } {
                            let task_id = task.id.clone();
                            
                            debug!(
                                "Worker {} thread {} processing task {}",
                                worker_id, thread_id, task_id
                            );

                            // Process the task
                            let result = Self::process_task(task).await;

                            match result {
                                Ok(processed_task) => {
                                    // Update task in storage
                                    if let Some(mut entry) = tasks.get_mut(&task_id) {
                                        *entry = processed_task;
                                    }
                                    tasks_processed.fetch_add(1, Ordering::Relaxed);
                                    debug!("Task {} processed successfully by worker {}", task_id, worker_id);
                                }
                                Err(e) => {
                                    error!("Task {} processing failed on worker {}: {}", task_id, worker_id, e);
                                    
                                    // Mark task as failed
                                    if let Some(mut entry) = tasks.get_mut(&task_id) {
                                        entry.set_failed(e.to_string());
                                    }
                                    tasks_failed.fetch_add(1, Ordering::Relaxed);
                                }
                            }
                        }
                    }
                    _ = tokio::time::sleep(Duration::from_millis(100)) => {
                        // Periodic check - prevents busy waiting
                    }
                }
            }

            info!("Processing thread {} stopped for worker {}", thread_id, worker_id);
        })
    }

    /// Process a single task
    async fn process_task(mut task: Task) -> Result<Task, TaskError> {
        let start_time = Instant::now();
        
        // Perform the calculation
        let result = Calculator::calculate(task.data.operation.clone(), task.data.input)?;
        
        let processing_time = start_time.elapsed();
        debug!(
            "Calculation completed in {:?}: {} {} = {}",
            processing_time,
            task.data.operation,
            task.data.input,
            result
        );

        // Update task status to processing (not completed - that requires API call)
        task.set_processing(result);
        
        Ok(task)
    }

}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::TaskData;
    use tokio::time::timeout;

    #[tokio::test]
    async fn test_worker_creation() {
        let worker = Worker::new(0, 2);
        assert_eq!(worker.id, 0);
        assert_eq!(worker.config.num_threads, 2);
    }

    #[tokio::test]
    async fn test_add_task() {
        let worker = Worker::new(0, 2);
        let task = Task::new(
            "Test task".to_string(),
            TaskPriority::High,
            TaskData::new(5, Operation::Factorial),
        );
        let task_id = task.id.clone();

        let result = worker.add_task(task).await;
        assert!(result.is_ok());

        let retrieved_task = worker.get_task(&task_id);
        assert!(retrieved_task.is_some());
        assert_eq!(retrieved_task.unwrap().id, task_id);
    }

    #[tokio::test]
    async fn test_task_processing() {
        let mut task = Task::new(
            "Test factorial".to_string(),
            TaskPriority::Medium,
            TaskData::new(5, Operation::Factorial),
        );

        let result = Worker::process_task(task.clone()).await;
        assert!(result.is_ok());

        let processed_task = result.unwrap();
        assert_eq!(processed_task.status, TaskStatus::Processing);
        assert_eq!(processed_task.result, Some("120".to_string()));
    }

    #[tokio::test]
    async fn test_worker_stats() {
        let worker = Worker::new(0, 2);
        let stats = worker.get_stats().await;
        
        assert_eq!(stats.id, 0);
        assert_eq!(stats.tasks_processed, 0);
        assert_eq!(stats.tasks_completed, 0);
        assert_eq!(stats.tasks_failed, 0);
    }
}
