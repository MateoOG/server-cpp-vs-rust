#![allow(warnings)]
use crate::types::{Operation, TaskError};
use tracing::debug;

/// Mathematical calculations module
/// 
/// This module provides implementations for the three supported operations:
/// - Factorial: calculates n!
/// - Fibonacci: calculates the nth Fibonacci number
/// - Prime check: determines if a number is prime
pub struct Calculator;

impl Calculator {
    /// Perform calculation based on operation type
    pub fn calculate(operation: Operation, input: u64) -> Result<String, TaskError> {
        debug!("Calculating {} for input {}", operation, input);
        
        let result = match operation {
            Operation::Factorial => Self::factorial(input)?,
            Operation::Fibonacci => Self::fibonacci(input)?,
            Operation::PrimeCheck => Self::prime_check(input)?,
        };
        
        debug!("Calculation result: {}", result);
        Ok(result)
    }

    /// Calculate factorial of n
    /// 
    /// Constraints: n <= 20 (to prevent overflow)
    /// Returns: n! as a string
    fn factorial(n: u64) -> Result<String, TaskError> {
        if n > 20 {
            return Err(TaskError::CalculationError {
                message: format!("Factorial input {} too large, maximum is 20", n),
            });
        }

        if n == 0 || n == 1 {
            return Ok("1".to_string());
        }

        // Use u128 to handle larger factorials safely
        let mut result: u128 = 1;
        for i in 2..=n {
            result = match result.checked_mul(i as u128) {
                Some(val) => val,
                None => {
                    return Err(TaskError::CalculationError {
                        message: format!("Factorial overflow for input {}", n),
                    });
                }
            };
        }

        Ok(result.to_string())
    }

    /// Calculate nth Fibonacci number
    /// 
    /// Constraints: n <= 93 (largest Fibonacci number that fits in u64)
    /// Returns: F(n) as a string
    fn fibonacci(n: u64) -> Result<String, TaskError> {
        if n > 93 {
            return Err(TaskError::CalculationError {
                message: format!("Fibonacci input {} too large, maximum is 93", n),
            });
        }

        match n {
            0 => Ok("0".to_string()),
            1 => Ok("1".to_string()),
            _ => {
                let mut a: u64 = 0;
                let mut b: u64 = 1;
                
                for _ in 2..=n {
                    let next = match a.checked_add(b) {
                        Some(val) => val,
                        None => {
                            return Err(TaskError::CalculationError {
                                message: format!("Fibonacci overflow for input {}", n),
                            });
                        }
                    };
                    a = b;
                    b = next;
                }
                
                Ok(b.to_string())
            }
        }
    }

    /// Check if a number is prime
    /// 
    /// Uses optimized trial division with early termination
    /// Returns: "true" if prime, "false" if not prime
    fn prime_check(n: u64) -> Result<String, TaskError> {
        if n < 2 {
            return Ok("false".to_string());
        }

        if n == 2 {
            return Ok("true".to_string());
        }

        if n % 2 == 0 {
            return Ok("false".to_string());
        }

        // Check odd divisors up to sqrt(n)
        let limit = ((n as f64).sqrt() as u64) + 1;
        for i in (3..=limit).step_by(2) {
            if n % i == 0 {
                return Ok("false".to_string());
            }
        }

        Ok("true".to_string())
    }

    /// Benchmark a calculation (for performance testing)
    #[cfg(test)]
    pub fn benchmark_calculation(operation: Operation, input: u64) -> Result<(String, std::time::Duration), TaskError> {
        let start = std::time::Instant::now();
        let result = Self::calculate(operation, input)?;
        let duration = start.elapsed();
        Ok((result, duration))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_factorial_basic() {
        assert_eq!(Calculator::factorial(0).unwrap(), "1");
        assert_eq!(Calculator::factorial(1).unwrap(), "1");
        assert_eq!(Calculator::factorial(5).unwrap(), "120");
        assert_eq!(Calculator::factorial(10).unwrap(), "3628800");
    }

    #[test]
    fn test_factorial_large() {
        // 20! = 2432902008176640000
        assert_eq!(Calculator::factorial(20).unwrap(), "2432902008176640000");
    }

    #[test]
    fn test_factorial_overflow() {
        let result = Calculator::factorial(21);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("too large"));
    }

    #[test]
    fn test_fibonacci_basic() {
        assert_eq!(Calculator::fibonacci(0).unwrap(), "0");
        assert_eq!(Calculator::fibonacci(1).unwrap(), "1");
        assert_eq!(Calculator::fibonacci(2).unwrap(), "1");
        assert_eq!(Calculator::fibonacci(10).unwrap(), "55");
        assert_eq!(Calculator::fibonacci(20).unwrap(), "6765");
    }

    #[test]
    fn test_fibonacci_large() {
        // F(50) = 12586269025
        assert_eq!(Calculator::fibonacci(50).unwrap(), "12586269025");
        // F(93) = 12200160415121876738 (largest that fits in u64)
        assert_eq!(Calculator::fibonacci(93).unwrap(), "12200160415121876738");
    }

    #[test]
    fn test_fibonacci_overflow() {
        let result = Calculator::fibonacci(94);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("too large"));
    }

    #[test]
    fn test_prime_check_basic() {
        assert_eq!(Calculator::prime_check(0).unwrap(), "false");
        assert_eq!(Calculator::prime_check(1).unwrap(), "false");
        assert_eq!(Calculator::prime_check(2).unwrap(), "true");
        assert_eq!(Calculator::prime_check(3).unwrap(), "true");
        assert_eq!(Calculator::prime_check(4).unwrap(), "false");
        assert_eq!(Calculator::prime_check(17).unwrap(), "true");
        assert_eq!(Calculator::prime_check(25).unwrap(), "false");
    }

    #[test]
    fn test_prime_check_large() {
        // Large prime: 982451653
        assert_eq!(Calculator::prime_check(982451653).unwrap(), "true");
        // Large composite: 982451654 = 2 × 491225827
        assert_eq!(Calculator::prime_check(982451654).unwrap(), "false");
    }

    #[test]
    fn test_calculate_integration() {
        // Test the main interface
        assert_eq!(
            Calculator::calculate(Operation::Factorial, 5).unwrap(),
            "120"
        );
        assert_eq!(
            Calculator::calculate(Operation::Fibonacci, 10).unwrap(),
            "55"
        );
        assert_eq!(
            Calculator::calculate(Operation::PrimeCheck, 17).unwrap(),
            "true"
        );
    }

    #[test]
    fn test_edge_cases() {
        // Test edge cases for each operation
        assert_eq!(Calculator::factorial(0).unwrap(), "1");
        assert_eq!(Calculator::fibonacci(0).unwrap(), "0");
        assert_eq!(Calculator::prime_check(2).unwrap(), "true");
    }

    #[test]
    fn test_performance() {
        use std::time::Duration;
        
        // Test that calculations complete within reasonable time
        let start = std::time::Instant::now();
        let _ = Calculator::factorial(20);
        assert!(start.elapsed() < Duration::from_millis(10));

        let start = std::time::Instant::now();
        let _ = Calculator::fibonacci(50);
        assert!(start.elapsed() < Duration::from_millis(10));

        let start = std::time::Instant::now();
        let _ = Calculator::prime_check(982451653);
        assert!(start.elapsed() < Duration::from_millis(100));
    }
}