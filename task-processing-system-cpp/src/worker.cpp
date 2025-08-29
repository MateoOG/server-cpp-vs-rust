/**
 * @file worker.cpp
 * @brief Implementation of Worker node using httplib with manual task completion
 */

#include "worker.h"
#include "calculations.h"
#include "httplib.h"
#include <iostream>
#include <sstream>
#include <iomanip>

using json = nlohmann::json;

nlohmann::json WorkerStats::to_json() const {
    auto now = std::chrono::steady_clock::now();
    auto uptime = std::chrono::duration_cast<std::chrono::seconds>(now - start_time).count();
    
    return json{
        {"tasks_processed", tasks_processed.load()},
        {"tasks_completed", tasks_completed.load()},
        {"tasks_failed", tasks_failed.load()},
        {"uptime_seconds", uptime}
    };
}

Worker::Worker(int worker_id, int thread_count)
    : worker_id_(worker_id), thread_count_(thread_count) {
    server_ = std::make_unique<httplib::Server>();
}

Worker::~Worker() {
    stop();
}

void Worker::start() {
    if (running_.load()) {
        return;
    }
    
    running_ = true;
    
    // Start processing threads
    worker_threads_.reserve(thread_count_);
    for (int i = 0; i < thread_count_; ++i) {
        worker_threads_.emplace_back(&Worker::processTaskLoop, this);
    }
    
    std::cout << "Worker " << worker_id_ << " with " << thread_count_ << " processing threads" << std::endl;
}

void Worker::stop() {
    if (!running_.load()) {
        return;
    }
    
    running_ = false;
    
    // Wake up all processing threads
    queue_cv_.notify_all();
    
    // Wait for processing threads to finish
    for (auto& thread : worker_threads_) {
        if (thread.joinable()) {
            thread.join();
        }
    }
    worker_threads_.clear();
    
    // Stop HTTP server
    if (server_) {
        server_->stop();
    }
  
    std::cout << "Worker " << worker_id_ << " stopped" << std::endl;
}

void Worker::addTask(const Task& task) {
    {
        std::lock_guard<std::mutex> lock(queue_mutex_);
        task_queue_.push(task);
    }
    
    {
        std::lock_guard<std::mutex> lock(storage_mutex_);
        task_storage_[task.getId()] = task;
    }
    
    queue_cv_.notify_one();
}

std::unique_ptr<Task> Worker::getTask(const std::string& task_id) {
    std::lock_guard<std::mutex> lock(storage_mutex_);
    auto it = task_storage_.find(task_id);
    if (it != task_storage_.end()) {
        return std::make_unique<Task>(it->second);
    }
    return nullptr;
}

bool Worker::completeTask(const std::string& task_id) {
    std::lock_guard<std::mutex> lock(storage_mutex_);
    auto it = task_storage_.find(task_id);
    if (it != task_storage_.end()) {
        Task& task = it->second;
        
        // Only allow completion of tasks that are in PROCESSING state and have a result
        if (task.getStatus() == TaskStatus::PROCESSING && !task.getResult().empty()) {
            // Simply change status to COMPLETED (calculation was already done)
            task.setStatus(TaskStatus::COMPLETED);
            stats_.tasks_completed++;
            return true;
        }
        
        // If task is in PROCESSING but has no result, it might have failed
        if (task.getStatus() == TaskStatus::PROCESSING && task.getResult().empty()) {
            // Check if it has an error message (failed during processing)
            if (!task.getErrorMessage().empty()) {
                task.setStatus(TaskStatus::FAILED);
                return false;
            }
        }
    }
    return false;
}

WorkerStats Worker::getStats() const {
    return stats_;
}

void Worker::processTaskLoop() {
    while (running_.load()) {
        std::unique_lock<std::mutex> lock(queue_mutex_);
        
        // Wait for task or stop signal
        queue_cv_.wait(lock, [this] { 
            return !task_queue_.empty() || !running_.load(); 
        });
        
        if (!running_.load()) {
            break;
        }
        
        if (task_queue_.empty()) {
            continue;
        }
        
        // FIFO
        Task task = task_queue_.front(); 
        task_queue_.pop();
        lock.unlock();
        
        // Process the task (only set to PROCESSING, don't complete automatically)
        processTask(task);
    }
}

void Worker::processTask(Task& task) {
    try {
        // Update task status to processing
        task.setStatus(TaskStatus::PROCESSING);
        
        {
            std::lock_guard<std::mutex> lock(storage_mutex_);
            task_storage_[task.getId()] = task;
        }
        
        // Execute calculation immediately (as before)
        std::string result = calculations::execute_calculation(
            task.getData().operation, 
            task.getData().input
        );
        
        // Store the result but keep status as PROCESSING
        // Task will remain in PROCESSING state until manually completed via API
        task.setResult(result);
        
        // Update task storage with result
        {
            std::lock_guard<std::mutex> lock(storage_mutex_);
            task_storage_[task.getId()] = task;
        }
        
        stats_.tasks_processed++;
        
    } catch (const std::exception& e) {
        // Task failed during calculation
        task.setErrorMessage(e.what());
        task.setStatus(TaskStatus::FAILED);
        
        {
            std::lock_guard<std::mutex> lock(storage_mutex_);
            task_storage_[task.getId()] = task;
        }
        
        stats_.tasks_failed++;
        stats_.tasks_processed++;
    }
}
