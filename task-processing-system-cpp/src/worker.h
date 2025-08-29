/**
 * @file worker.h
 * @brief Worker node for task processing with HTTP server using cpp-httplib
 * @author mnog
 * @date 2025
 */

#ifndef WORKER_H
#define WORKER_H

#include "task.h"
#include <memory>
#include <thread>
#include <atomic>
#include <queue>
#include <mutex>
#include <condition_variable>
#include <unordered_map>
#include <iostream>

// Forward declaration for HTTP server (cpp-httplib)
namespace httplib {
    class Server;
}

/**
 * @brief Statistics for a worker
 */
struct WorkerStats {
    std::atomic<uint64_t> tasks_processed{0};    ///< Total tasks processed
    std::atomic<uint64_t> tasks_completed{0};    ///< Successfully completed tasks
    std::atomic<uint64_t> tasks_failed{0};       ///< Failed tasks
    std::chrono::steady_clock::time_point start_time; ///< Worker start time
    
    WorkerStats() : start_time(std::chrono::steady_clock::now()) {}
    
    // Copy constructor
    WorkerStats(const WorkerStats& other) 
        : tasks_processed(other.tasks_processed.load()),
          tasks_completed(other.tasks_completed.load()),
          tasks_failed(other.tasks_failed.load()),
          start_time(other.start_time) {}
    
    // Assignment operator
    WorkerStats& operator=(const WorkerStats& other) {
        if (this != &other) {
            tasks_processed.store(other.tasks_processed.load());
            tasks_completed.store(other.tasks_completed.load());
            tasks_failed.store(other.tasks_failed.load());
            start_time = other.start_time;
        }
        return *this;
    }
    
    /**
     * @brief Convert stats to JSON
     * @return JSON representation of statistics
     */
    nlohmann::json to_json() const;
};

/**
 * @brief Worker node that processes tasks and provides HTTP API
 */
class Worker {
private:
    int worker_id_;                                    ///< Unique worker identifier
    int thread_count_;                                 ///< Number of processing threads
    std::atomic<bool> running_{false};                 ///< Worker running state
    
    // Task management
    std::queue<Task> task_queue_;
    std::mutex queue_mutex_;                           ///< Queue protection mutex
    std::condition_variable queue_cv_;                 ///< Queue condition variable
    std::unordered_map<std::string, Task> task_storage_; ///< Task storage by ID
    std::mutex storage_mutex_;                         ///< Storage protection mutex
    
    // Threading
    std::vector<std::thread> worker_threads_;          ///< Processing threads
    std::thread http_server_thread_;                   ///< HTTP server thread
    
    // Statistics
    WorkerStats stats_;                                ///< Worker statistics
    
    // HTTP Server (using cpp-httplib)
    std::unique_ptr<httplib::Server> server_;          ///< HTTP server instance
    
    /**
     * @brief Main processing loop for worker threads
     */
    void processTaskLoop();
    
    /**
     * @brief Process a single task (executes calculation but keeps status as PROCESSING)
     * @param task Task to process
     */
    void processTask(Task& task);
    
    /**
     * @brief Setup HTTP routes for the worker API
     */
    void setupRoutes();
    
    /**
     * @brief HTTP server main loop
     */
    void runHttpServer();

public:
    /**
     * @brief Constructor
     * @param worker_id Unique worker identifier
     * @param thread_count Number of processing threads
     */
    Worker(int worker_id, int thread_count);
    
    /**
     * @brief Destructor
     */
    ~Worker();
    
    /**
     * @brief Start the worker (processing threads and HTTP server)
     */
    void start();
    
    /**
     * @brief Stop the worker gracefully
     */
    void stop();
    
    /**
     * @brief Add task to processing queue
     * @param task Task to add
     */
    void addTask(const Task& task);
    
    /**
     * @brief Get task by ID
     * @param task_id Task identifier
     * @return Task if found, nullptr otherwise
     */
    std::unique_ptr<Task> getTask(const std::string& task_id);
    
    /**
     * @brief Manually complete a task via API call
     * @param task_id Task identifier
     * @return true if task was found and successfully completed
     */
    bool completeTask(const std::string& task_id);
    
    /**
     * @brief Get worker statistics
     * @return Current worker statistics
     */
    WorkerStats getStats() const;
    
    /**
     * @brief Get worker ID
     * @return Worker identifier
     */
    int getWorkerId() const { return worker_id_; }
    
    /**
     * @brief Check if worker is running
     * @return true if running, false otherwise
     */
    bool isRunning() const { return running_.load(); }
};

#endif // WORKER_H
