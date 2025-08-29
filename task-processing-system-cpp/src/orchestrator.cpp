/**
 * @file orchestrator.cpp
 * @brief Implementation of Task Orchestrator
 */

#include "orchestrator.h"
#include "httplib.h"
#include <iostream>
#include <algorithm>
#include <thread>
#include <chrono>
#include "httplib.h"
#include <iostream>
#include <algorithm>
#include <thread>

using json = nlohmann::json;

nlohmann::json SystemStats::to_json() const {
    auto now = std::chrono::steady_clock::now();
    auto uptime = std::chrono::duration_cast<std::chrono::seconds>(now - start_time).count();
    
    return json{
        {"total_tasks_processed", total_tasks_processed},
        {"total_tasks_completed", total_tasks_completed},
        {"total_tasks_failed", total_tasks_failed},
        {"total_workers", total_workers},
        {"uptime_seconds", uptime}
    };
}

OrchestratorConfig OrchestratorConfig::from_json(const nlohmann::json& j) {
    OrchestratorConfig config;
    if (j.contains("num_workers")) config.num_workers = j["num_workers"];
    if (j.contains("threads_per_worker")) config.threads_per_worker = j["threads_per_worker"];
    if (j.contains("orchestrator_port")) config.orchestrator_port = j["orchestrator_port"];
    return config;
}

nlohmann::json OrchestratorConfig::to_json() const {
    return json{
        {"num_workers", num_workers},
        {"threads_per_worker", threads_per_worker},
        {"orchestrator_port", orchestrator_port}
    };
}

TaskOrchestrator::TaskOrchestrator(const OrchestratorConfig& config) 
    : config_(config) {
    server_ = std::make_unique<httplib::Server>();
    setupRoutes();
    
    // Create workers
    workers_.reserve(config_.num_workers);
    for (int i = 0; i < config_.num_workers; ++i) {
        workers_.emplace_back(
            std::make_unique<Worker>(i, config_.threads_per_worker)
        );
    }
    
    system_stats_.total_workers = config_.num_workers;
}

TaskOrchestrator::~TaskOrchestrator() {
    stop();
}

void TaskOrchestrator::start() {
    if (running_.load()) {
        return;
    }
    
    running_ = true;
    
    // Start all workers
    for (auto& worker : workers_) {
        worker->start();
    }
    
    // Start orchestrator HTTP server
    http_server_thread_ = std::thread(&TaskOrchestrator::runHttpServer, this);
    
    std::cout << "Task Orchestrator started with " << config_.num_workers 
              << " workers on port " << config_.orchestrator_port << std::endl;
    
    // Print worker information
    for (const auto& worker : workers_) {
        std::cout << "  Worker ID" << worker->getWorkerId() << std::endl;
    }
}

void TaskOrchestrator::stop() {
    if (!running_.load()) {
        return;
    }
    
    running_ = false;
    
    // Stop all workers
    for (auto& worker : workers_) {
        worker->stop();
    }
    
    // Stop orchestrator HTTP server
    if (server_) {
        server_->stop();
    }
    
    if (http_server_thread_.joinable()) {
        http_server_thread_.join();
    }
    
    std::cout << "Task Orchestrator stopped" << std::endl;
}

void TaskOrchestrator::runHttpServer() {
    try {
        std::cout << "Starting Orchestrator HTTP server on port " << config_.orchestrator_port << std::endl;
        server_->listen("0.0.0.0", config_.orchestrator_port);
    } catch (const std::exception& e) {
        std::cerr << "Orchestrator HTTP Server error on port " 
                  << config_.orchestrator_port << ": " << e.what() << std::endl;
    }
}

int TaskOrchestrator::distributeTask(const Task& task) {
    if (workers_.empty()) {
        throw std::runtime_error("No workers available");
    }
    
    // Select worker using round-robin
    size_t worker_index = selectWorker();
    workers_[worker_index]->addTask(task);
    
    std::cout << "Distributed task " << task.getId() 
              << " (priority: " << static_cast<int>(task.getPriority()) 
              << ") to worker " << worker_index << std::endl;
    
    return static_cast<int>(worker_index);
}

std::string TaskOrchestrator::createTask(const Task& task) {
    // Validate input before accepting task
    if (!validateTaskInput(task)) {
        throw std::invalid_argument("Invalid task input");
    }
    
    distributeTask(task);
    
    return task.getId();
}

std::unique_ptr<Task> TaskOrchestrator::getTask(const std::string& task_id) {
    // Search all workers for the task
    for (auto& worker : workers_) {
        auto task = worker->getTask(task_id);
        if (task) {
            return task;
        }
    }
    return nullptr;
}

bool TaskOrchestrator::completeTask(const std::string& task_id) {
    // Try to complete task on all workers (using worker's own completeTask method)
    for (auto& worker : workers_) {
        if (worker->completeTask(task_id)) {
            return true;
        }
    }
    return false;
}

SystemStats TaskOrchestrator::getSystemStats() {
    updateSystemStats();
    return system_stats_;
}

size_t TaskOrchestrator::selectWorker() {
    // Round-robin selection for workers
    size_t worker_index = current_worker_.fetch_add(1) % workers_.size();
    return worker_index;
}

void TaskOrchestrator::updateSystemStats() {
    system_stats_.total_tasks_processed = 0;
    system_stats_.total_tasks_completed = 0;
    system_stats_.total_tasks_failed = 0;
    
    for (const auto& worker : workers_) {
        WorkerStats stats = worker->getStats();
        system_stats_.total_tasks_processed += stats.tasks_processed.load();
        system_stats_.total_tasks_completed += stats.tasks_completed.load();
        system_stats_.total_tasks_failed += stats.tasks_failed.load();
    }
}


bool TaskOrchestrator::validateTaskInput(const Task& task) const {
    // Validate using existing Task::isValid() method
    if (!task.isValid()) {
        return false;
    }
    
    // Additional orchestrator-level validations
    const auto& data = task.getData();
    
    // Check if operation is supported
    if (data.operation != "factorial" && 
        data.operation != "fibonacci" && 
        data.operation != "prime_check") {
        return false;
    }
    
    // Check input ranges for specific operations
    if (data.operation == "factorial" && (data.input < 0 || data.input > 20)) {
        return false; // Factorial limited to reasonable range
    }
    
    if (data.operation == "fibonacci" && (data.input < 0 || data.input > 1000)) {
        return false; // Fibonacci limited to reasonable range
    }
    
    if (data.operation == "prime_check" && data.input < 2) {
        return false; // Prime check requires input >= 2
    }
    
    return true;
}

void TaskOrchestrator::setupRoutes() {
    // POST /task/create - Create new task
    server_->Post("/task/create", [this](const httplib::Request& req, httplib::Response& res) {
        try {
            json input = json::parse(req.body);
            
            // Create task from JSON
            Task task = Task::from_json(input);
            
            // Validate and create task
            std::string task_id = createTask(task);
            
            json response = {
                {"message", "Task created successfully"},
                {"task_id", task_id},
                {"status", "pending"}
            };
            
            res.status = 200;
            res.body = response.dump();
            
        } catch (const std::invalid_argument& e) {
            res.status = 400;
            res.body = json{{"error", "Invalid input: " + std::string(e.what())}}.dump();
        } catch (const std::exception& e) {
            res.status = 500;
            res.body = json{{"error", "Internal server error: " + std::string(e.what())}}.dump();
        }
        res.set_header("Content-Type", "application/json");
    });
    
    // GET /task/{id} - Get task information
    server_->Get(R"(/task/([^/]+))", [this](const httplib::Request& req, httplib::Response& res) {
        std::string task_id = req.matches[1];
        
        auto task = getTask(task_id);
        if (task) {
            res.status = 200;
            res.body = task->to_json().dump();
        } else {
            res.status = 404;
            res.body = json{{"error", "Task not found"}}.dump();
        }
        res.set_header("Content-Type", "application/json");
    });
    
    // POST /task/{id}/complete - Mark task as completed (only way to complete)
    server_->Post(R"(/task/([^/]+)/complete)", [this](const httplib::Request& req, httplib::Response& res) {
        std::string task_id = req.matches[1];
        
        bool completed = completeTask(task_id);
        if (completed) {
            // Get updated task to return with result
            auto task = getTask(task_id);
            json response = {
                {"message", "Task marked as completed"},
                {"task_id", task_id},
                {"status", "completed"}
            };
            
            if (task && !task->getResult().empty()) {
                response["result"] = task->getResult();
            }
            
            res.status = 200;
            res.body = response.dump();
        } else {
            // Check if task exists but cannot be completed
            auto task = getTask(task_id);
            if (task) {
                std::string current_status = taskStatusToString(task->getStatus());
                res.status = 400;
                res.body = json{
                    {"error", "Task cannot be completed"},
                    {"task_id", task_id},
                    {"current_status", current_status},
                    {"reason", "Task must be in processing state with result to be completed"}
                }.dump();
            } else {
                res.status = 404;
                res.body = json{{"error", "Task not found"}}.dump();
            }
        }
        res.set_header("Content-Type", "application/json");
    });
    
    // GET /stats - Get system statistics
    server_->Get("/stats", [this](const httplib::Request& , httplib::Response& res) {
        SystemStats stats = getSystemStats();
        json response = stats.to_json();
        
        // Add worker details
        std::vector<json> worker_stats;
        for (size_t i = 0; i < workers_.size(); ++i) {
            json worker_stat = workers_[i]->getStats().to_json();
            worker_stat["worker_id"] = static_cast<int>(i);
            worker_stats.push_back(worker_stat);
        }
        response["workers"] = worker_stats;
        
        res.status = 200;
        res.body = response.dump();
        res.set_header("Content-Type", "application/json");
    });
}