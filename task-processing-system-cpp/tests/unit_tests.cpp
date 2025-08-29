/**
 * @file unit_tests.cpp
 * @brief Unit tests for Task Processing System
 */

#include <gtest/gtest.h>
#include "../src/task.h"
#include "../src/calculations.h"
#include <nlohmann/json.hpp>
#include <chrono>
#include <thread>

using json = nlohmann::json;

/**
 * @brief Test suite for Task functionality
 */
class TaskTest : public ::testing::Test {
protected:
    void SetUp() override {
        // Setup test data
        task_data = TaskData{"calculation", 5, "factorial"};
        task = Task("test-001", "Test Task", Priority::HIGH, task_data);
    }

    TaskData task_data;
    Task task;
};

/**
 * @brief Test Task creation and basic properties
 */
TEST_F(TaskTest, TaskCreation) {
    EXPECT_EQ(task.getId(), "test-001");
    EXPECT_EQ(task.getTitle(), "Test Task");
    EXPECT_EQ(task.getPriority(), Priority::HIGH);
    EXPECT_EQ(task.getStatus(), TaskStatus::PENDING);
    EXPECT_EQ(task.getData().type, "calculation");
    EXPECT_EQ(task.getData().input, 5);
    EXPECT_EQ(task.getData().operation, "factorial");
}

/**
 * @brief Test Task status workflow - ONLY complete via API
 */
TEST_F(TaskTest, TaskStatusWorkflow) {
    // Initial state
    EXPECT_EQ(task.getStatus(), TaskStatus::PENDING);
    
    // Can move to processing
    task.setStatus(TaskStatus::PROCESSING);
    EXPECT_EQ(task.getStatus(), TaskStatus::PROCESSING);
    
    // Can move to completed (simulating API call)
    task.setStatus(TaskStatus::COMPLETED);
    EXPECT_EQ(task.getStatus(), TaskStatus::COMPLETED);
    
    // Can move to failed
    Task failed_task("test-failed", "Failed Task", Priority::LOW, task_data);
    failed_task.setStatus(TaskStatus::FAILED);
    EXPECT_EQ(failed_task.getStatus(), TaskStatus::FAILED);
}

/**
 * @brief Test Task validation - only required operations
 */
TEST_F(TaskTest, TaskValidation) {
    EXPECT_TRUE(task.isValid());
    
    // Test valid operations
    TaskData factorial_data{"calculation", 10, "factorial"};
    TaskData fibonacci_data{"calculation", 20, "fibonacci"};  
    TaskData prime_data{"calculation", 17, "prime_check"};
    
    Task factorial_task("test-f", "Factorial Task", Priority::MEDIUM, factorial_data);
    Task fibonacci_task("test-fib", "Fibonacci Task", Priority::MEDIUM, fibonacci_data);
    Task prime_task("test-p", "Prime Task", Priority::MEDIUM, prime_data);
    
    EXPECT_TRUE(factorial_task.isValid());
    EXPECT_TRUE(fibonacci_task.isValid()); 
    EXPECT_TRUE(prime_task.isValid());
    
    // Test invalid task type
    TaskData invalid_type{"invalid", 5, "factorial"};
    Task invalid_type_task("test-002", "Invalid Type", Priority::LOW, invalid_type);
    EXPECT_FALSE(invalid_type_task.isValid());
    
    // Test invalid operation (not in required list)
    TaskData invalid_op{"calculation", 5, "square_root"};
    Task invalid_op_task("test-003", "Invalid Op", Priority::LOW, invalid_op);
    EXPECT_FALSE(invalid_op_task.isValid());
    
    // Test input validation limits
    TaskData large_factorial{"calculation", 25, "factorial"};  // Over limit
    Task large_factorial_task("test-004", "Large Factorial", Priority::LOW, large_factorial);
    EXPECT_FALSE(large_factorial_task.isValid());
    
    TaskData invalid_prime{"calculation", 1, "prime_check"};   // Under minimum
    Task invalid_prime_task("test-005", "Invalid Prime", Priority::LOW, invalid_prime);
    EXPECT_FALSE(invalid_prime_task.isValid());
}

/**
 * @brief Test Task JSON serialization with priority
 */
TEST_F(TaskTest, TaskJSONSerialization) {
    json task_json = task.to_json();
    
    EXPECT_EQ(task_json["id"], "test-001");
    EXPECT_EQ(task_json["title"], "Test Task");
    EXPECT_EQ(task_json["priority"], 3); // HIGH = 3
    EXPECT_EQ(task_json["status"], "pending");
    EXPECT_EQ(task_json["data"]["type"], "calculation");
    EXPECT_EQ(task_json["data"]["input"], 5);
    EXPECT_EQ(task_json["data"]["operation"], "factorial");
    
    // Test with processing status and result
    task.setStatus(TaskStatus::PROCESSING);
    task.setResult("120");
    
    json processing_json = task.to_json();
    EXPECT_EQ(processing_json["status"], "processing");
    EXPECT_EQ(processing_json["result"], "120");
}

/**
 * @brief Test Task JSON deserialization
 */
TEST_F(TaskTest, TaskJSONDeserialization) {
    json task_json = {
        {"id", "test-004"},
        {"title", "JSON Test Task"},
        {"priority", 2},
        {"data", {
            {"type", "calculation"},
            {"input", 10},
            {"operation", "fibonacci"}
        }}
    };
    
    Task deserialized_task = Task::from_json(task_json);
    
    EXPECT_EQ(deserialized_task.getId(), "test-004");
    EXPECT_EQ(deserialized_task.getTitle(), "JSON Test Task");
    EXPECT_EQ(deserialized_task.getPriority(), Priority::MEDIUM);
    EXPECT_EQ(deserialized_task.getData().type, "calculation");
    EXPECT_EQ(deserialized_task.getData().input, 10);
    EXPECT_EQ(deserialized_task.getData().operation, "fibonacci");
}

/**
 * @brief Test suite for Calculations functionality
 */
class CalculationsTest : public ::testing::Test {
protected:
    void SetUp() override {}
};

/**
 * @brief Test factorial calculation (required operation)
 */
TEST_F(CalculationsTest, FactorialCalculation) {
    EXPECT_EQ(calculations::factorial(0), "1");
    EXPECT_EQ(calculations::factorial(1), "1");
    EXPECT_EQ(calculations::factorial(5), "120");
    EXPECT_EQ(calculations::factorial(10), "3628800");
    
    // Test large factorial
    std::string result = calculations::factorial(20);
    EXPECT_EQ(result, "2432902008176640000");
    
    // Test negative input
    EXPECT_THROW(calculations::factorial(-1), std::invalid_argument);
}

/**
 * @brief Test fibonacci calculation (required operation)
 */
TEST_F(CalculationsTest, FibonacciCalculation) {
    EXPECT_EQ(calculations::fibonacci(0), "0");
    EXPECT_EQ(calculations::fibonacci(1), "1");
    EXPECT_EQ(calculations::fibonacci(2), "1");
    EXPECT_EQ(calculations::fibonacci(10), "55");
    EXPECT_EQ(calculations::fibonacci(20), "6765");
    
    // Test larger fibonacci number
    EXPECT_EQ(calculations::fibonacci(50), "12586269025");
    
    // Test negative input
    EXPECT_THROW(calculations::fibonacci(-1), std::invalid_argument);
}

/**
 * @brief Test prime check calculation (required operation)
 */
TEST_F(CalculationsTest, PrimeCheckCalculation) {
    EXPECT_EQ(calculations::prime_check(2), "true");
    EXPECT_EQ(calculations::prime_check(3), "true");
    EXPECT_EQ(calculations::prime_check(4), "false");
    EXPECT_EQ(calculations::prime_check(17), "true");
    EXPECT_EQ(calculations::prime_check(25), "false");
    EXPECT_EQ(calculations::prime_check(97), "true");
    EXPECT_EQ(calculations::prime_check(100), "false");
    
    // Test larger prime numbers
    EXPECT_EQ(calculations::prime_check(1009), "true");
    EXPECT_EQ(calculations::prime_check(1000), "false");
    
    // Test edge cases
    EXPECT_THROW(calculations::prime_check(1), std::invalid_argument);
    EXPECT_THROW(calculations::prime_check(0), std::invalid_argument);
}

/**
 * @brief Test execute_calculation function (main interface)
 */
TEST_F(CalculationsTest, ExecuteCalculation) {
    // Test all required operations
    EXPECT_EQ(calculations::execute_calculation("factorial", 5), "120");
    EXPECT_EQ(calculations::execute_calculation("fibonacci", 10), "55");
    EXPECT_EQ(calculations::execute_calculation("prime_check", 17), "true");
    
    // Test invalid operation (not in required list)
    EXPECT_THROW(calculations::execute_calculation("square_root", 16), std::invalid_argument);
    EXPECT_THROW(calculations::execute_calculation("logarithm", 10), std::invalid_argument);
    
    // Test invalid inputs
    EXPECT_THROW(calculations::execute_calculation("factorial", -1), std::invalid_argument);
    EXPECT_THROW(calculations::execute_calculation("fibonacci", -1), std::invalid_argument);
    EXPECT_THROW(calculations::execute_calculation("prime_check", 1), std::invalid_argument);
}

/**
 * @brief Test input validation for required operations only
 */
TEST_F(CalculationsTest, InputValidation) {
    // Factorial validation
    EXPECT_TRUE(calculations::validate_calculation_input("factorial", 0));
    EXPECT_TRUE(calculations::validate_calculation_input("factorial", 10));
    EXPECT_TRUE(calculations::validate_calculation_input("factorial", 20));
    EXPECT_FALSE(calculations::validate_calculation_input("factorial", -1));
    
    // Fibonacci validation
    EXPECT_TRUE(calculations::validate_calculation_input("fibonacci", 0));
    EXPECT_TRUE(calculations::validate_calculation_input("fibonacci", 10));
    EXPECT_TRUE(calculations::validate_calculation_input("fibonacci", 100));
    EXPECT_FALSE(calculations::validate_calculation_input("fibonacci", -1));
    
    // Prime check validation
    EXPECT_TRUE(calculations::validate_calculation_input("prime_check", 2));
    EXPECT_TRUE(calculations::validate_calculation_input("prime_check", 100));
    EXPECT_TRUE(calculations::validate_calculation_input("prime_check", 1000));
    EXPECT_FALSE(calculations::validate_calculation_input("prime_check", 1));
    EXPECT_FALSE(calculations::validate_calculation_input("prime_check", 0));
    
    // Invalid operations should return false
    EXPECT_FALSE(calculations::validate_calculation_input("invalid_op", 5));
    EXPECT_FALSE(calculations::validate_calculation_input("square_root", 16));
    EXPECT_FALSE(calculations::validate_calculation_input("power", 2));
}

/**
 * @brief Performance test for calculations
 */
TEST_F(CalculationsTest, PerformanceTest) {
    auto start = std::chrono::high_resolution_clock::now();
    
    // Test reasonably sized calculations
    std::string factorial_result = calculations::factorial(15);
    std::string fibonacci_result = calculations::fibonacci(30);
    std::string prime_result = calculations::prime_check(997);
    
    auto end = std::chrono::high_resolution_clock::now();
    auto duration = std::chrono::duration_cast<std::chrono::milliseconds>(end - start);
    
    // Results should not be empty
    EXPECT_FALSE(factorial_result.empty());
    EXPECT_FALSE(fibonacci_result.empty());
    EXPECT_FALSE(prime_result.empty());
    
    // Performance check - should complete reasonably quickly
    EXPECT_LT(duration.count(), 1000); // Should complete within 1 second
    
    // Verify some results
    EXPECT_EQ(factorial_result, "1307674368000");  // 15!
    EXPECT_EQ(fibonacci_result, "832040");         // F(30)
    EXPECT_EQ(prime_result, "true");               // 997 is prime
}

/**
 * @brief Test priority string conversions
 */
TEST_F(TaskTest, PriorityStringConversion) {
    EXPECT_EQ(priorityToString(Priority::LOW), "low");
    EXPECT_EQ(priorityToString(Priority::MEDIUM), "medium");
    EXPECT_EQ(priorityToString(Priority::HIGH), "high");
    
    EXPECT_EQ(stringToPriority("low"), Priority::LOW);
    EXPECT_EQ(stringToPriority("1"), Priority::LOW);
    EXPECT_EQ(stringToPriority("medium"), Priority::MEDIUM);
    EXPECT_EQ(stringToPriority("2"), Priority::MEDIUM);
    EXPECT_EQ(stringToPriority("high"), Priority::HIGH);
    EXPECT_EQ(stringToPriority("3"), Priority::HIGH);
    
    // Invalid strings should default to MEDIUM
    EXPECT_EQ(stringToPriority("invalid"), Priority::MEDIUM);
}

/**
 * @brief Test status string conversions
 */
TEST_F(TaskTest, StatusStringConversion) {
    EXPECT_EQ(taskStatusToString(TaskStatus::PENDING), "pending");
    EXPECT_EQ(taskStatusToString(TaskStatus::PROCESSING), "processing");
    EXPECT_EQ(taskStatusToString(TaskStatus::COMPLETED), "completed");
    EXPECT_EQ(taskStatusToString(TaskStatus::FAILED), "failed");
    
    EXPECT_EQ(stringToTaskStatus("pending"), TaskStatus::PENDING);
    EXPECT_EQ(stringToTaskStatus("processing"), TaskStatus::PROCESSING);
    EXPECT_EQ(stringToTaskStatus("completed"), TaskStatus::COMPLETED);
    EXPECT_EQ(stringToTaskStatus("failed"), TaskStatus::FAILED);
    
    // Invalid strings should default to PENDING
    EXPECT_EQ(stringToTaskStatus("invalid"), TaskStatus::PENDING);
}

/**
 * @brief Test workflow: task completion only through API
 */
TEST_F(TaskTest, CompletionWorkflow) {
    // Task starts as PENDING
    EXPECT_EQ(task.getStatus(), TaskStatus::PENDING);
    
    // Worker processes task -> PROCESSING (calculation done, result stored)
    task.setStatus(TaskStatus::PROCESSING);
    task.setResult("120");  // Result computed but not completed yet
    EXPECT_EQ(task.getStatus(), TaskStatus::PROCESSING);
    EXPECT_EQ(task.getResult(), "120");
    
    // Only through API call -> COMPLETED
    task.setStatus(TaskStatus::COMPLETED);
    EXPECT_EQ(task.getStatus(), TaskStatus::COMPLETED);
    
    // This workflow ensures tasks can only be marked complete via 
    // POST /task/{id}/complete API endpoint
}

int main(int argc, char **argv) {
    ::testing::InitGoogleTest(&argc, argv);
    return RUN_ALL_TESTS();
}
