/**
 * @file task.cpp
 * @brief Implementation of Task data structures
 */

#include "task.h"
#include <iomanip>
#include <sstream>
#include <stdexcept>

using json = nlohmann::json;

nlohmann::json TaskData::to_json() const {
    return json{
        {"type", type},
        {"input", input},
        {"operation", operation}
    };
}

TaskData TaskData::from_json(const nlohmann::json& j) {
    TaskData data;
    try {
        data.type = j.at("type").get<std::string>();
        data.input = j.at("input").get<int>();
        data.operation = j.at("operation").get<std::string>();
    } catch (const std::exception& e) {
        throw std::invalid_argument("Invalid TaskData JSON: " + std::string(e.what()));
    }
    return data;
}

Task::Task(const std::string& id, const std::string& title, Priority priority, const TaskData& data)
    : id_(id), title_(title), priority_(priority), data_(data), status_(TaskStatus::PENDING) {
    created_at_ = std::chrono::system_clock::now();
}

nlohmann::json Task::to_json() const {
    auto time_t = std::chrono::system_clock::to_time_t(created_at_);
    std::stringstream ss;
    ss << std::put_time(std::gmtime(&time_t), "%Y-%m-%dT%H:%M:%SZ");
    
    json j = {
        {"id", id_},
        {"title", title_},
        {"priority", static_cast<int>(priority_)},
        {"created_at", ss.str()},
        {"data", data_.to_json()},
        {"status", taskStatusToString(status_)}
    };
    
    if (!result_.empty()) {
        j["result"] = result_;
    }
    
    if (!error_message_.empty()) {
        j["error"] = error_message_;
    }
    
    return j;
}

Task Task::from_json(const nlohmann::json& j) {
    try {
        std::string id = j.at("id").get<std::string>();
        std::string title = j.at("title").get<std::string>();
        
        // Handle priority - default to MEDIUM if not specified
        Priority priority = Priority::MEDIUM;
        if (j.contains("priority")) {
            int priority_value = j.at("priority").get<int>();
            if (priority_value >= 1 && priority_value <= 3) {
                priority = static_cast<Priority>(priority_value);
            }
        }
        
        TaskData data = TaskData::from_json(j.at("data"));
        
        Task task(id, title, priority, data);
        
        // Set status if provided
        if (j.contains("status")) {
            task.status_ = stringToTaskStatus(j.at("status").get<std::string>());
        }
        
        // Set result if provided
        if (j.contains("result")) {
            task.result_ = j.at("result").get<std::string>();
        }
        
        // Set error message if provided
        if (j.contains("error")) {
            task.error_message_ = j.at("error").get<std::string>();
        }
        
        return task;
    } catch (const std::exception& e) {
        throw std::invalid_argument("Invalid Task JSON: " + std::string(e.what()));
    }
}

bool Task::isValid() const {
    // Check required fields
    if (id_.empty() || title_.empty()) {
        return false;
    }
    
    // Validate task data
    if (data_.type != "calculation") {
        return false;
    }
    
    // Check supported operations
    if (data_.operation != "factorial" && 
        data_.operation != "fibonacci" && 
        data_.operation != "prime_check") {
        return false;
    }
    
    // Validate input range (reasonable limits)
    if (data_.input < 0 || data_.input > 100000) {
        return false;
    }
    
    // Additional operation-specific validation
    if (data_.operation == "factorial" && data_.input > 20) {
        return false; // Factorial limited to prevent overflow
    }
    
    if (data_.operation == "fibonacci" && data_.input > 1000) {
        return false; // Fibonacci limited to reasonable range
    }
    
    if (data_.operation == "prime_check" && data_.input < 2) {
        return false; // Prime check requires input >= 2
    }
    
    return true;
}

std::string taskStatusToString(TaskStatus status) {
    switch (status) {
        case TaskStatus::PENDING:
            return "pending";
        case TaskStatus::PROCESSING:
            return "processing";
        case TaskStatus::COMPLETED:
            return "completed";
        case TaskStatus::FAILED:
            return "failed";
        default:
            return "unknown";
    }
}

TaskStatus stringToTaskStatus(const std::string& status) {
    if (status == "pending") {
        return TaskStatus::PENDING;
    } else if (status == "processing") {
        return TaskStatus::PROCESSING;
    } else if (status == "completed") {
        return TaskStatus::COMPLETED;
    } else if (status == "failed") {
        return TaskStatus::FAILED;
    }
    return TaskStatus::PENDING;
}

std::string priorityToString(Priority priority) {
    switch (priority) {
        case Priority::LOW:
            return "low";
        case Priority::MEDIUM:
            return "medium";
        case Priority::HIGH:
            return "high";
        default:
            return "medium";
    }
}

Priority stringToPriority(const std::string& priority) {
    if (priority == "low" || priority == "1") {
        return Priority::LOW;
    } else if (priority == "medium" || priority == "2") {
        return Priority::MEDIUM;
    } else if (priority == "high" || priority == "3") {
        return Priority::HIGH;
    }
    return Priority::MEDIUM;
}
