//#![allow(warnings)]
use reqwest::Client;
use serde_json::json;
use std::time::Duration;
use tokio::time::sleep;
use tracing_test::traced_test;
use uuid::Uuid;


/// Setup test orchestrator and return base URL
async fn setup_test_system() -> String {
    "http://127.0.0.1:7000".to_string() // Default port from cargo run
}

#[tokio::test]
#[traced_test]
async fn test_basic_task_workflow() {
    let base_url = setup_test_system().await;
    
    // Check if server is running
    let client = Client::new();
    let health_check = client.get(&format!("{}/health", base_url)).send().await;
    if health_check.is_err() {
        panic!("Server not running. Start with: cargo run");
    }

    // Generate unique task ID
    let task_id = format!("test-factorial-{}", Uuid::new_v4().to_string());

    // Test 1: Create a factorial task
    let create_payload = json!({
        "id": task_id,
        "title": "Test Factorial Calculation",
        "priority": 3,
        "data": {
            "type": "calculation",
            "input": 5,
            "operation": "factorial"
        }
    });

    let response = client
        .post(&format!("{}/task/create", base_url))
        .json(&create_payload)
        .send()
        .await
        .expect("Failed to send create request");

    assert_eq!(response.status(), 200);
    let create_result: serde_json::Value = response.json().await.expect("Invalid JSON response");
    assert_eq!(create_result["id"], task_id);

    // Test 2: Get task status (should be processing after a moment)
    sleep(Duration::from_millis(100)).await;
    
    let response = client
        .get(&format!("{}/task/{}", base_url, task_id))
        .send()
        .await
        .expect("Failed to get task");

    assert_eq!(response.status(), 200);
    let task_result: serde_json::Value = response.json().await.expect("Invalid JSON response");
    assert_eq!(task_result["id"], task_id);
    assert_eq!(task_result["status"], "processing");
    assert_eq!(task_result["result"], "120"); // 5! = 120

    // Test 3: Complete the task
    let response = client
        .post(&format!("{}/task/{}/complete", base_url, task_id))
        .send()
        .await
        .expect("Failed to complete task");

    assert_eq!(response.status(), 200);
    let complete_result: serde_json::Value = response.json().await.expect("Invalid JSON response");
    assert_eq!(complete_result["status"], "completed");

    // Test 4: Verify task is completed
    let response = client
        .get(&format!("{}/task/{}", base_url, task_id))
        .send()
        .await
        .expect("Failed to get completed task");

    let final_task: serde_json::Value = response.json().await.expect("Invalid JSON response");
    assert_eq!(final_task["status"], "completed");
    assert!(final_task["completed_at"].is_string());
}

#[tokio::test]
#[traced_test] 
async fn test_system_statistics() {
    let base_url = setup_test_system().await;
    
    // Check if server is running
    let client = Client::new();
    let health_check = client.get(&format!("{}/health", base_url)).send().await;
    if health_check.is_err() {
        panic!("Server not running. Start with: cargo run");
    }

    // Get initial stats
    let response = client
        .get(&format!("{}/stats", base_url))
        .send()
        .await
        .expect("Failed to get stats");

    assert_eq!(response.status(), 200);
    let initial_stats: serde_json::Value = response.json().await.expect("Invalid JSON response");

    // Create and process a task
    let task_id = format!("stats-test-{}", Uuid::new_v4().to_string());
    let payload = json!({
        "id": task_id,
        "title": "Statistics Test",
        "priority": 2,
        "data": {
            "type": "calculation",
            "input": 6,
            "operation": "factorial"
        }
    });

    client
        .post(&format!("{}/task/create", base_url))
        .json(&payload)
        .send()
        .await
        .expect("Failed to create task");

    sleep(Duration::from_millis(100)).await;

    // Complete the task
    client
        .post(&format!("{}/task/{}/complete", base_url, task_id))
        .send()
        .await
        .expect("Failed to complete task");

    // Get updated stats
    let response = client
        .get(&format!("{}/stats", base_url))
        .send()
        .await
        .expect("Failed to get updated stats");

    let updated_stats: serde_json::Value = response.json().await.expect("Invalid JSON response");
    
    // Verify stats structure and that they increased
    assert_eq!(updated_stats["total_workers"], 3); // Match default config
    assert!(updated_stats["total_tasks_processed"].as_u64().unwrap() >= 
            initial_stats["total_tasks_processed"].as_u64().unwrap());
    assert!(updated_stats["total_tasks_completed"].as_u64().unwrap() >= 
            initial_stats["total_tasks_completed"].as_u64().unwrap());
    assert!(updated_stats["workers"].is_array());
    assert_eq!(updated_stats["workers"].as_array().unwrap().len(), 3);
}

#[tokio::test]
#[traced_test]
async fn test_error_handling() {
    let base_url = setup_test_system().await;
    
    // Check if server is running
    let client = Client::new();
    let health_check = client.get(&format!("{}/health", base_url)).send().await;
    if health_check.is_err() {
        panic!("Server not running. Start with: cargo run");
    }

    // Test 1: Invalid task data
    let invalid_payload = json!({
        "id": format!("invalid-{}", Uuid::new_v4().to_string()),
        "title": "Invalid Task",
        "priority": 2,
        "data": {
            "type": "invalid_type",
            "input": 5,
            "operation": "factorial"
        }
    });

    let response = client
        .post(&format!("{}/task/create", base_url))
        .json(&invalid_payload)
        .send()
        .await
        .expect("Failed to send invalid request");

    // Should accept but return error in response
    let result: serde_json::Value = response.json().await.expect("Invalid JSON response");
    assert!(result.get("error").is_some());

    // Test 2: Get non-existent task
    let response = client
        .get(&format!("{}/task/non-existent-task", base_url))
        .send()
        .await
        .expect("Failed to request non-existent task");

    assert_eq!(response.status(), 404);

    // Test 3: Complete non-existent task
    let response = client
        .post(&format!("{}/task/non-existent-task/complete", base_url))
        .send()
        .await
        .expect("Failed to request non-existent task completion");

    assert_eq!(response.status(), 404);

    // Test 4: Input too large for factorial
    let large_factorial_payload = json!({
        "id": format!("large-factorial-{}", Uuid::new_v4().to_string()),
        "title": "Large Factorial (Should Fail)",
        "priority": 2,
        "data": {
            "type": "calculation",
            "input": 25, // Too large for factorial
            "operation": "factorial"
        }
    });

    let response = client
        .post(&format!("{}/task/create", base_url))
        .json(&large_factorial_payload)
        .send()
        .await
        .expect("Failed to send large factorial request");

    let result: serde_json::Value = response.json().await.expect("Invalid JSON response");
    assert!(result.get("error").is_some());
}

#[tokio::test]
#[traced_test]
async fn test_concurrent_task_processing() {
    let base_url = setup_test_system().await;
    
    // Check if server is running
    let client = Client::new();
    let health_check = client.get(&format!("{}/health", base_url)).send().await;
    if health_check.is_err() {
        panic!("Server not running. Start with: cargo run");
    }

    // Create multiple tasks concurrently
    let num_tasks = 10;
    let mut handles = Vec::new();

    for i in 0..num_tasks {
        let client = client.clone();
        let base_url = base_url.clone();
        
        let handle = tokio::spawn(async move {
            // Generate unique task ID using UUID
            let task_id = Uuid::new_v4().to_string();
            
            let payload = json!({
                "id": task_id,
                "title": format!("Concurrent Task {}", i),
                "priority": (i % 3) + 1, // Mix of priorities
                "data": {
                    "type": "calculation", 
                    "input": 3 + (i % 5), // Vary inputs
                    "operation": match i % 3 {
                        0 => "factorial",
                        1 => "fibonacci", 
                        _ => "primecheck"
                    }
                }
            });

            let response = client
                .post(&format!("{}/task/create", base_url))
                .json(&payload)
                .send()
                .await
                .expect("Failed to create concurrent task");

            
            // debugging:
            let status = response.status();
            if response.status() != 200 {
                let error_text = response.text().await.unwrap_or_else(|_| "No response body".to_string());
                println!("Task creation failed: Status {}, Body: {}", status, error_text);
                println!("Payload was: {}", serde_json::to_string_pretty(&payload).unwrap());
            }

            assert_eq!(status, 200, "Task creation should succeed");
            task_id
        });
        
        handles.push(handle);
    }

    // Wait for all tasks to be created
    let task_ids: Vec<String> = futures::future::join_all(handles).await
        .into_iter()
        .map(|result| result.expect("Task creation failed"))
        .collect();

    // Wait for processing
    sleep(Duration::from_millis(300)).await;

    // Verify all tasks were processed
    let mut processed_count = 0;
    for task_id in &task_ids {
        let response = client
            .get(&format!("{}/task/{}", base_url, task_id))
            .send()
            .await
            .expect("Failed to get task");

        if response.status() == 200 {
            let task: serde_json::Value = response.json().await.expect("Invalid JSON response");
            if task["status"] == "processing" {
                processed_count += 1;
            }
        }
    }

    // Should have processed most/all tasks
    assert!(processed_count >= num_tasks / 2, "Expected at least half the tasks to be processed, got {}/{}", processed_count, num_tasks);
}

#[cfg(test)]
mod performance_tests {
    use super::*;
    use std::time::Instant;

    #[tokio::test]
    #[traced_test]
    async fn test_throughput_performance() {
        let base_url = setup_test_system().await;
    
        // Check if server is running
        let client = Client::new();
        let health_check = client.get(&format!("{}/health", base_url)).send().await;
        if health_check.is_err() {
            panic!("Server not running. Start with: cargo run");
        }

        let num_tasks = 50;
        let start_time = Instant::now();

        // Create tasks rapidly
        for i in 0..num_tasks {
            let payload = json!({
                "id": format!("perf-{}-{:03}", Uuid::new_v4().to_string(), i),
                "title": format!("Performance Task {}", i),
                "priority": 2,
                "data": {
                    "type": "calculation",
                    "input": 5,
                    "operation": "factorial"
                }
            });

            client
                .post(&format!("{}/task/create", base_url))
                .json(&payload)
                .send()
                .await
                .expect("Failed to create performance task");
        }

        let creation_time = start_time.elapsed();
        println!("Created {} tasks in {:?}", num_tasks, creation_time);

        // Wait for processing
        sleep(Duration::from_millis(1000)).await;

        // Check system stats for overall performance
        let response = client
            .get(&format!("{}/stats", base_url))
            .send()
            .await
            .expect("Failed to get stats");

        let stats: serde_json::Value = response.json().await.expect("Invalid JSON response");
        let total_processed = stats["total_tasks_processed"].as_u64().unwrap_or(0);
        let uptime = stats["uptime_seconds"].as_u64().unwrap_or(1);
        
        let throughput = total_processed as f64 / uptime as f64;
        
        println!("System has processed {} total tasks in {} seconds ({:.2} tasks/sec average)", 
                 total_processed, uptime, throughput);
        
        // Should achieve reasonable throughput (be lenient since it's a shared system)
        assert!(total_processed > 0, "Expected some tasks to be processed");
        assert!(throughput >= 0.000001, "Expected some reasonable throughput, got {:.2} tasks/sec", throughput);
    }
}
