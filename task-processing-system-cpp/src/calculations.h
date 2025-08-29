/**
 * @file calculations.h
 * @brief Mathematical calculation operations for task processing
 * @author mnog
 * @date 2025
 */

#ifndef CALCULATIONS_H
#define CALCULATIONS_H

#include <string>
#include <cstdint>

/**
 * @brief Mathematical calculations namespace
 */
namespace calculations {

/**
 * @brief Calculate factorial of a number
 * @param n The number to calculate factorial for (must be >= 0)
 * @return Factorial of n as string (to handle large numbers)
 * @throws std::invalid_argument if n < 0
 */
std::string factorial(int n);

/**
 * @brief Calculate nth Fibonacci number
 * @param n The position in Fibonacci sequence (must be >= 0)
 * @return nth Fibonacci number as string (to handle large numbers)
 * @throws std::invalid_argument if n < 0
 */
std::string fibonacci(int n);

/**
 * @brief Check if a number is prime
 * @param n The number to check (must be >= 2)
 * @return "true" if prime, "false" if not prime
 * @throws std::invalid_argument if n < 2
 */
std::string prime_check(int n);

/**
 * @brief Execute calculation based on operation type
 * @param operation The operation type (factorial, fibonacci, prime_check)
 * @param input The input value
 * @return Result as string
 * @throws std::invalid_argument for invalid operations or inputs
 */
std::string execute_calculation(const std::string& operation, int input);

/**
 * @brief Validate if operation and input are compatible
 * @param operation The operation type
 * @param input The input value
 * @return true if valid, false otherwise
 */
bool validate_calculation_input(const std::string& operation, int input);

} // namespace calculations

#endif // CALCULATIONS_H