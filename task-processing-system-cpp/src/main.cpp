/**
 * @file main.cpp
 * @brief Main entry point for the Task Processing System
 * @author mnog
 * @date 2025
 */

#include <iostream>
#include <string>
#include <vector>
#include <filesystem>
#include <fstream>
#include <csignal>
#include <atomic>
#include <memory>

#include "orchestrator.h"
#include "task.h"

// Global variables for signal handling
std::atomic<bool> shutdown_requested{false};
std::unique_ptr<TaskOrchestrator> global_orchestrator = nullptr;

// Forward declarations
void printUsage(const char* program_name);

/**
 * @brief Signal handler for graceful shutdown
 * @param signal Signal number
 */
void signalHandler(int signal) {
    std::cout << "\nReceived signal " << signal << ". Initiating graceful shutdown..." << std::endl;
    shutdown_requested = true;
    
    if (global_orchestrator) {
        global_orchestrator->stop();
    }
}

/**
 * @brief Parse command line arguments
 * @param argc Argument count
 * @param argv Argument vector
 * @return Configuration object
 */
OrchestratorConfig parseCommandLine(int argc, char* argv[]) {
    OrchestratorConfig config;
    
    for (int i = 1; i < argc; i++) {
        std::string arg(argv[i]);
        
        if (arg == "--help" || arg == "-h") {
            printUsage(argv[0]);
            exit(0);
        } else if (arg == "--workers" || arg == "-w") {
            if (i + 1 < argc) {
                config.num_workers = std::stoi(argv[++i]);
                if (config.num_workers <= 0 || config.num_workers > 50) {
                    throw std::invalid_argument("Number of workers must be between 1 and 50");
                }
            } else {
                throw std::invalid_argument("--workers requires a value");
            }
        } else if (arg == "--threads" || arg == "-t") {
            if (i + 1 < argc) {
                config.threads_per_worker = std::stoi(argv[++i]);
                if (config.threads_per_worker <= 0 || config.threads_per_worker > 32) {
                    throw std::invalid_argument("Threads per worker must be between 1 and 32");
                }
            } else {
                throw std::invalid_argument("--threads requires a value");
            }
        } else if (arg == "--orchestrator-port" || arg == "-o") {
            if (i + 1 < argc) {
                config.orchestrator_port = std::stoi(argv[++i]);
                if (config.orchestrator_port <= 1024 || config.orchestrator_port > 65535) {
                    throw std::invalid_argument("Orchestrator port must be between 1025 and 65535");
                }
            } else {
                throw std::invalid_argument("--orchestrator-port requires a value");
            }
        } else if (arg == "--config" || arg == "-c") {
            if (i + 1 < argc) {
                std::string config_file = argv[++i];
                if (!std::filesystem::exists(config_file)) {
                    throw std::invalid_argument("Configuration file does not exist: " + config_file);
                }
                
                std::ifstream file(config_file);
                nlohmann::json j;
                file >> j;
                config = OrchestratorConfig::from_json(j);
            } else {
                throw std::invalid_argument("--config requires a file path");
            }
        } else {
            throw std::invalid_argument("Unknown argument: " + arg);
        }
    }
    
    return config;
}

/**
 * @brief Print usage information
 * @param program_name Name of the program
 */
void printUsage(const char* program_name) {
    std::cout << "Usage: " << program_name << " [OPTIONS]" << std::endl;
    std::cout << std::endl;
    std::cout << "Task Processing System - A distributed function processor" << std::endl;
    std::cout << std::endl;
    std::cout << "Options:" << std::endl;
    std::cout << "  -w, --workers NUM          Number of worker nodes (default: 3, max: 50)" << std::endl;
    std::cout << "  -t, --threads NUM          Threads per worker (default: 4, max: 32)" << std::endl;
    std::cout << "  -o, --orchestrator-port NUM Orchestrator port (default: 5000)" << std::endl;
    std::cout << "  -c, --config FILE          Configuration file (JSON)" << std::endl;
    std::cout << "  -h, --help                 Show this help message" << std::endl;
    std::cout << std::endl;
    std::cout << "Configuration File Format (JSON):" << std::endl;
    std::cout << "{" << std::endl;
    std::cout << "  \"num_workers\": 3," << std::endl;
    std::cout << "  \"threads_per_worker\": 4," << std::endl;
    std::cout << "  \"orchestrator_port\": 5000" << std::endl;
    std::cout << "}" << std::endl;
    std::cout << std::endl;
    std::cout << "Example:" << std::endl;
    std::cout << "  " << program_name << " --workers 5 --threads 8 " << std::endl;
}

/**
 * @brief Print system information with only required API endpoints
 */
void printSystemInfo(const OrchestratorConfig& config) {
    std::cout << "=== Task Processing System ===" << std::endl;
    std::cout << "Configuration:" << std::endl;
    std::cout << "  Workers: " << config.num_workers << std::endl;
    std::cout << "  Threads per worker: " << config.threads_per_worker << std::endl;
    std::cout << "  Orchestrator port: " << config.orchestrator_port << std::endl;
    std::cout << std::endl;
    
    std::cout << "API Endpoints (Required Only):" << std::endl;
    std::cout << "  Orchestrator: http://localhost:" << config.orchestrator_port << std::endl;
    std::cout << "    POST /task/create        - Create a new task" << std::endl;
    std::cout << "    GET /task/{id}           - Get task information" << std::endl;
    std::cout << "    POST /task/{id}/complete - Mark task as completed (ONLY way to complete)" << std::endl;
    std::cout << "    GET /stats               - Get worker statistics" << std::endl;
    std::cout << std::endl;
    
    std::cout << std::endl;
    
    std::cout << "Supported Operations:" << std::endl;
    std::cout << "  - factorial: Calculate factorial of input" << std::endl;
    std::cout << "  - fibonacci: Calculate nth Fibonacci number" << std::endl;
    std::cout << "  - prime_check: Check if input is prime" << std::endl;
    std::cout << std::endl;
    
    std::cout << "Task Priority:" << std::endl;
    std::cout << "  1 = LOW priority" << std::endl;
    std::cout << "  2 = MEDIUM priority (default)" << std::endl;
    std::cout << "  3 = HIGH priority" << std::endl;
    std::cout << std::endl;
    
    std::cout << "Task Completion Workflow:" << std::endl;
    std::cout << "  1. Task created -> STATUS: pending" << std::endl;
    std::cout << "  2. Worker processes -> STATUS: processing (calculation done)" << std::endl;
    std::cout << "  3. API call to complete -> STATUS: completed" << std::endl;
    std::cout << "  Tasks can ONLY be marked completed via POST /task/{id}/complete" << std::endl;
    std::cout << std::endl;
    
    std::cout << "JSON Format Example:" << std::endl;
    std::cout << "{" << std::endl;
    std::cout << "  \"id\": \"task-001\"," << std::endl;
    std::cout << "  \"title\": \"Process calculation\"," << std::endl;
    std::cout << "  \"priority\": 3," << std::endl;
    std::cout << "  \"data\": {" << std::endl;
    std::cout << "    \"type\": \"calculation\"," << std::endl;
    std::cout << "    \"input\": 10," << std::endl;
    std::cout << "    \"operation\": \"factorial\"" << std::endl;
    std::cout << "  }" << std::endl;
    std::cout << "}" << std::endl;
    std::cout << std::endl;
}

/**
 * @brief Validate configuration
 * @param config Configuration to validate
 */
void validateConfiguration(const OrchestratorConfig& config) {
    // Check port conflicts
    
    // Check reasonable limits
    if (config.num_workers * config.threads_per_worker > 200) {
        std::cout << "Warning: High total thread count (" 
                  << (config.num_workers * config.threads_per_worker) 
                  << "). This may impact performance." << std::endl;
    }
    
    std::cout << "Configuration validated successfully." << std::endl;
}

/**
 * @brief Wait for shutdown signal
 */
void waitForShutdown() {
    while (!shutdown_requested.load()) {
        std::this_thread::sleep_for(std::chrono::milliseconds(100));
    }
}

/**
 * @brief Main entry point
 * @param argc Argument count
 * @param argv Argument vector
 * @return Exit code
 */
int main(int argc, char* argv[]) {
    try {
        // Setup signal handlers for graceful shutdown
        signal(SIGINT, signalHandler);
        signal(SIGTERM, signalHandler);
        
        // Parse configuration
        OrchestratorConfig config = parseCommandLine(argc, argv);
        validateConfiguration(config);
        
        // Print system information
        printSystemInfo(config);
        
        // Create and start orchestrator
        global_orchestrator = std::make_unique<TaskOrchestrator>(config);
        global_orchestrator->start();
        
        std::cout << "Task Processing System started successfully!" << std::endl;
        std::cout << "Tasks must be completed via POST /task/{id}/complete API call." << std::endl;
        std::cout << "Press Ctrl+C to shutdown gracefully..." << std::endl;
        std::cout << std::string(50, '=') << std::endl;
        
        // Wait for shutdown signal
        waitForShutdown();
        
        std::cout << "Shutting down..." << std::endl;
        global_orchestrator->stop();
        global_orchestrator.reset();
        
        std::cout << "Task Processing System shutdown complete." << std::endl;
        return 0;
        
    } catch (const std::invalid_argument& e) {
        std::cerr << "Configuration Error: " << e.what() << std::endl;
        std::cerr << "Use --help for usage information." << std::endl;
        return 1;
    } catch (const std::exception& e) {
        std::cerr << "Error: " << e.what() << std::endl;
        return 1;
    } catch (...) {
        std::cerr << "Unknown error occurred." << std::endl;
        return 1;
    }
}
