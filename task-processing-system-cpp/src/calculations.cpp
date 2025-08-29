/**
 * @file calculations.cpp
 * @brief Implementation of mathematical calculation operations
 * 
 * @date 2025
 */

#include "calculations.h"
#include <stdexcept>
#include <vector>
#include <cmath>
#include <algorithm>

namespace calculations {

std::string factorial(int n) {
    if (n < 0) {
        throw std::invalid_argument("Factorial is not defined for negative numbers");
    }
    
    if (n == 0 || n == 1) {
        return "1";
    }
    
    // Use vector to store digits for large number multiplication
    std::vector<int> result(1, 1);  // Initialize with 1
    
    for (int i = 2; i <= n; i++) {
        int carry = 0;
        for (size_t j = 0; j < result.size(); j++) {
            int prod = result[j] * i + carry;
            result[j] = prod % 10;
            carry = prod / 10;
        }
        
        while (carry) {
            result.push_back(carry % 10);
            carry /= 10;
        }
    }
    
    // Convert result to string (reverse order)
    std::string factorial_str;
    factorial_str.reserve(result.size());
    for (int i = result.size() - 1; i >= 0; i--) {
        factorial_str += std::to_string(result[i]);
    }
    
    return factorial_str;
}

// Forward declaration of helper function
static std::string addStrings(const std::string& num1, const std::string& num2);

std::string fibonacci(int n) {
    if (n < 0) {
        throw std::invalid_argument("Fibonacci is not defined for negative numbers");
    }
    
    if (n == 0) return "0";
    if (n == 1) return "1";
    
    // Use string arithmetic for large Fibonacci numbers
    std::string prev = "0";
    std::string curr = "1";
    
    for (int i = 2; i <= n; i++) {
        std::string next = addStrings(prev, curr);
        prev = curr;
        curr = next;
    }
    
    return curr;
}

std::string prime_check(int n) {
    if (n < 2) {
        throw std::invalid_argument("Prime check requires number >= 2");
    }
    
    if (n == 2) return "true";
    if (n % 2 == 0) return "false";
    
    // Check odd divisors up to sqrt(n)
    int limit = static_cast<int>(std::sqrt(n));
    for (int i = 3; i <= limit; i += 2) {
        if (n % i == 0) {
            return "false";
        }
    }
    
    return "true";
}

std::string execute_calculation(const std::string& operation, int input) {
    if (!validate_calculation_input(operation, input)) {
        throw std::invalid_argument("Invalid operation or input");
    }
    
    if (operation == "factorial") {
        return factorial(input);
    } else if (operation == "fibonacci") {
        return fibonacci(input);
    } else if (operation == "prime_check") {
        return prime_check(input);
    } else {
        throw std::invalid_argument("Unknown operation: " + operation);
    }
}

bool validate_calculation_input(const std::string& operation, int input) {
    if (operation == "factorial") {
        return input >= 0;
    } else if (operation == "fibonacci") {
        return input >= 0;
    } else if (operation == "prime_check") {
        return input >= 2;
    }
    return false;
}

// Helper function to add two number strings  
static std::string addStrings(const std::string& num1, const std::string& num2) {
    std::string result;
    int carry = 0;
    int i = num1.length() - 1;
    int j = num2.length() - 1;
    
    while (i >= 0 || j >= 0 || carry) {
        int digit1 = (i >= 0) ? num1[i] - '0' : 0;
        int digit2 = (j >= 0) ? num2[j] - '0' : 0;
        
        int sum = digit1 + digit2 + carry;
        result = std::to_string(sum % 10) + result;
        carry = sum / 10;
        
        i--;
        j--;
    }
    
    return result;
}

} // namespace calculations
