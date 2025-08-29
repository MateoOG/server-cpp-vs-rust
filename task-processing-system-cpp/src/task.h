/**
 * @file task.h
 * @brief Task data structures and types for the function processor
 * @author mnog
 * @date 2025
 */

#ifndef TASK_H
#define TASK_H

#include <string>
#include <chrono>
#include <nlohmann/json.hpp>

/**
 * @brief Task priority levels (higher number = higher priority)
 */
enum class Priority : int {
    LOW = 1,    ///< Low priority
    MEDIUM = 2, ///< Medium priority
    HIGH = 3    ///< High priority
};

/**
 * @brief Task status enumeration
 */
enum class TaskStatus {
    PENDING,    ///< Task is waiting to be processed
    PROCESSING, ///< Task is currently being processed (calculation done, awaiting completion)
    COMPLETED,  ///< Task has been completed successfully via API call
    FAILED      ///< Task processing failed
};

/**
 * @brief Task data structure containing operation parameters
 */
struct TaskData {
    std::string type;      ///< Type of calculation (calculation)
    int input;             ///< Input value for the operation
    std::string operation; ///< Operation type (factorial, fibonacci, prime_check)
    
    /**
     * @brief Convert TaskData to JSON
     */
    nlohmann::json to_json() const;
    
    /**
     * @brief Create TaskData from JSON
     */
    static TaskData from_json(const nlohmann::json& j);
};

/**
 * @brief Main Task structure
 */
class Task {
private:
    std::string id_;
    std::string title_;
    Priority priority_;
    std::chrono::system_clock::time_point created_at_;
    TaskData data_;
    TaskStatus status_;
    std::string result_;
    std::string error_message_;

public:
    /**
     * @brief Default constructor
     */
    Task() = default;
    
    /**
     * @brief Constructor with parameters
     */
    Task(const std::string& id, const std::string& title, Priority priority, 
         const TaskData& data);
    
    // Getters
    const std::string& getId() const { return id_; }
    const std::string& getTitle() const { return title_; }
    Priority getPriority() const { return priority_; }
    const std::chrono::system_clock::time_point& getCreatedAt() const { return created_at_; }
    const TaskData& getData() const { return data_; }
    TaskStatus getStatus() const { return status_; }
    const std::string& getResult() const { return result_; }
    const std::string& getErrorMessage() const { return error_message_; }
    
    // Setters
    void setStatus(TaskStatus status) { status_ = status; }
    void setResult(const std::string& result) { result_ = result; }
    void setErrorMessage(const std::string& error) { error_message_ = error; }
    
    /**
     * @brief Convert Task to JSON representation
     */
    nlohmann::json to_json() const;
    
    /**
     * @brief Create Task from JSON
     */
    static Task from_json(const nlohmann::json& j);
    
    /**
     * @brief Validate task data
     */
    bool isValid() const;
};

/**
 * @brief Task comparator for worker priority queue (higher priority first, then FIFO)
 */
struct TaskComparator {
    bool operator()(const Task& a, const Task& b) const {
        // Higher priority values have higher priority
        // If priorities are equal, compare by creation time (FIFO)
        if (static_cast<int>(a.getPriority()) != static_cast<int>(b.getPriority())) {
            return static_cast<int>(a.getPriority()) < static_cast<int>(b.getPriority());
        }
        return a.getCreatedAt() > b.getCreatedAt();
    }
};

/**
 * @brief Convert TaskStatus to string
 */
std::string taskStatusToString(TaskStatus status);

/**
 * @brief Convert string to TaskStatus
 */
TaskStatus stringToTaskStatus(const std::string& status);

/**
 * @brief Convert Priority to string
 */
std::string priorityToString(Priority priority);

/**
 * @brief Convert string to Priority
 */
Priority stringToPriority(const std::string& priority);

#endif // TASK_H
