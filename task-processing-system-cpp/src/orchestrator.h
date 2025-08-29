/**
 * @file orchestrator.h
 * @brief Task orchestrator with round-robin load balancing
 * @author mnog
 * @date 2025
 */

#ifndef ORCHESTRATOR_H
#define ORCHESTRATOR_H

#include "worker.h"
#include "task.h"
#include <vector>
#include <memory>
#include <atomic>
#include <thread>
#include <iostream>
#include <queue>
#include <mutex>
#include <condition_variable>

// Forward declaration for HTTP server
namespace httplib {
    class Server;
}

/**
 * @brief System-wide statistics
 */
struct SystemStats {
    uint64_t total_tasks_processed = 0;    ///< Total tasks across all workers
    uint64_t total_tasks_completed = 0;    ///< Total completed tasks
    uint64_t total_tasks_failed = 0;       ///< Total failed tasks
    uint64_t total_workers = 0;            ///< Number of workers
    std::chrono::steady_clock::time_point start_time; ///< System start time
    
    SystemStats() : start_time(std::chrono::steady_clock::now()) {}
    
    /**
     * @brief Convert system stats to JSON
     */
    nlohmann::json to_json() const;
};

/**
 * @brief Configuration for the orchestrator
 */
struct OrchestratorConfig {
    int num_workers = 3;                   ///< Number of workers
    int threads_per_worker = 4;            ///< Threads per worker
    int orchestrator_port = 5000;          ///< Orchestrator API port
    
    /**
     * @brief Load configuration from JSON
     */
    static OrchestratorConfig from_json(const nlohmann::json& j);
    
    /**
     * @brief Convert configuration to JSON
     */
    nlohmann::json to_json() const;
};

/**
 * @brief Task orchestrator that manages multiple workers with round-robin distribution
 */
class TaskOrchestrator {
private:
    OrchestratorConfig config_;                        ///< Configuration
    std::vector<std::unique_ptr<Worker>> workers_;     ///< Worker instances
    std::atomic<size_t> current_worker_{0};            ///< Round-robin counter
    std::atomic<bool> running_{false};                 ///< Orchestrator running state
    
    // HTTP Server for orchestrator API
    std::unique_ptr<httplib::Server> server_;          ///< HTTP server
    std::thread http_server_thread_;                   ///< HTTP server thread
    
    // Statistics
    SystemStats system_stats_;                         ///< System statistics
    
    // Priority queue for task distribution (orchestrator level)
    std::mutex queue_mutex_;                           ///< Mutex for pending tasks queue
    
    /**
     * @brief Setup HTTP routes for orchestrator
     */
    void setupRoutes();
    
    /**
     * @brief Run orchestrator HTTP server
     */
    void runHttpServer();
    
    /**
     * @brief Select next worker using round-robin for same priority
     * @return Index of selected worker
     */
    size_t selectWorker();
    
    /**
     * @brief Update system statistics
     */
    void updateSystemStats();
    
    /**
     * @brief Validate task input data
     * @param task Task to validate
     * @return true if valid, false otherwise
     */
    bool validateTaskInput(const Task& task) const;

    /**
     * @brief Distribute a task to a worker based on the selected policy.
     * @param task The task to distribute.
     * @return The index of the worker the task was distributed to.
     */
    int distributeTask(const Task& task);

public:
    /**
     * @brief Constructor with configuration
     * @param config Orchestrator configuration
     */
    explicit TaskOrchestrator(const OrchestratorConfig& config);
    
    /**
     * @brief Destructor
     */
    ~TaskOrchestrator();
    
    /**
     * @brief Start all workers and orchestrator
     */
    void start();
    
    /**
     * @brief Stop all workers and orchestrator
     */
    void stop();
    
    /**
     * @brief Create and distribute task
     * @param task Task to create
     * @return Task ID if accepted
     */
    std::string createTask(const Task& task);
    
    /**
     * @brief Get task from any worker
     * @param task_id Task identifier
     * @return Task if found, nullptr otherwise
     */
    std::unique_ptr<Task> getTask(const std::string& task_id);
    
    /**
     * @brief Complete task on appropriate worker (only way to mark as completed)
     * @param task_id Task identifier
     * @return true if task was found and marked as completed
     */
    bool completeTask(const std::string& task_id);
    
    /**
     * @brief Get system statistics
     * @return Current system statistics
     */
    SystemStats getSystemStats();
    
    /**
     * @brief Get configuration
     * @return Current configuration
     */
    const OrchestratorConfig& getConfig() const { return config_; }
    
    /**
     * @brief Check if orchestrator is running
     * @return true if running, false otherwise
     */
    bool isRunning() const { return running_.load(); }
};

#endif // ORCHESTRATOR_H