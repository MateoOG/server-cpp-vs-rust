use reqwest::Client;
use serde_json::json;
use std::error::Error;
use tokio::time::{sleep, Duration};

/// Example client demonstrating how to use the Task Processing System API
#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    println!("=== Task Processing System Client Example ===\n");

    let client = Client::new();
    let base_url = "http://localhost:7000";

    // Check if system is healthy
    println!("1. Checking system health...");
    match check_health(&client, base_url).await {
        Ok(()) => println!(" System is healthy\n"),
        Err(e) => {
            eprintln!(" System health check failed: {}", e);
            eprintln!("Make sure the task processing system is running!");
            eprintln!("Run: cargo run");
            return Ok(());
        }
    }

    // Example 1: High-priority factorial task
    println!("2. Creating high-priority factorial task...");
    let task_id = create_factorial_task(&client, base_url).await?;
    println!(" Created task: {}\n", task_id);

    // Wait for processing
    println!("3. Waiting for task processing...");
    sleep(Duration::from_millis(500)).await;

    // Check task status
    println!("4. Checking task status...");
    check_task_status(&client, base_url, &task_id).await?;

    // Complete the task
    println!("5. Completing the task...");
    complete_task(&client, base_url, &task_id).await?;

    // Verify completion
    println!("6. Verifying task completion...");
    check_task_status(&client, base_url, &task_id).await?;

    // Example 2: Create multiple tasks with different priorities
    println!("7. Creating multiple tasks with different priorities...");
    create_multiple_tasks(&client, base_url).await?;

    // Check system statistics
    println!("8. Getting system statistics...");
    get_system_stats(&client, base_url).await?;

    println!("\n=== Example completed successfully! ===");
    Ok(())
}

/// Check system health
async fn check_health(client: &Client, base_url: &str) -> Result<(), Box<dyn Error>> {
    let response = client
        .get(&format!("{}/health", base_url))
        .send()
        .await?;

    if response.status().is_success() {
        let health: serde_json::Value = response.json().await?;
        println!("Status: {}", health["status"]);
        Ok(())
    } else {
        Err(format!("Health check failed: {}", response.status()).into())
    }
}

/// Create a factorial task
async fn create_factorial_task(client: &Client, base_url: &str) -> Result<String, Box<dyn Error>> {
    let task_payload = json!({
        "id": "example-factorial-001",
        "title": "Calculate 10! (factorial)",
        "priority": 3,
        "data": {
            "type": "calculation",
            "input": 10,
            "operation": "factorial"
        }
    });

    let response = client
        .post(&format!("{}/task/create", base_url))
        .json(&task_payload)
        .send()
        .await?;

    let result: serde_json::Value = response.json().await?;
    
    if let Some(error) = result.get("error") {
        return Err(format!("Task creation failed: {}", error).into());
    }

    Ok(result["id"].as_str().unwrap_or("unknown").to_string())
}

/// Check task status
async fn check_task_status(client: &Client, base_url: &str, task_id: &str) -> Result<(), Box<dyn Error>> {
    let response = client
        .get(&format!("{}/task/{}", base_url, task_id))
        .send()
        .await?;

    if response.status().is_success() {
        let task: serde_json::Value = response.json().await?;
        println!("Task Status:");
        println!("  ID: {}", task["id"]);
        println!("  Title: {}", task["title"]);
        println!("  Status: {}", task["status"]);
        println!("  Priority: {}", task["priority"]);
        
        if let Some(result) = task.get("result") {
            println!("  Result: {}", result);
        }
        
        if let Some(completed_at) = task.get("completed_at") {
            println!("  Completed: {}", completed_at);
        }
        println!();
    } else {
        println!(" Task not found or error occurred");
    }

    Ok(())
}

/// Complete a task
async fn complete_task(client: &Client, base_url: &str, task_id: &str) -> Result<(), Box<dyn Error>> {
    let response = client
        .post(&format!("{}/task/{}/complete", base_url, task_id))
        .send()
        .await?;

    if response.status().is_success() {
        let result: serde_json::Value = response.json().await?;
        println!(" Task completed: {}", result["message"]);
    } else {
        println!(" Failed to complete task");
    }

    Ok(())
}

/// Create multiple tasks with different priorities
async fn create_multiple_tasks(client: &Client, base_url: &str) -> Result<(), Box<dyn Error>> {
    let tasks = vec![
        ("fibonacci-low", 1, 15, "fibonacci"),
        ("prime-high", 3, 97, "prime_check"),
        ("factorial-medium", 2, 7, "factorial"),
    ];

    for (id, priority, input, operation) in tasks {
        let payload = json!({
            "id": format!("example-{}", id),
            "title": format!("Example {} task", operation),
            "priority": priority,
            "data": {
                "type": "calculation",
                "input": input,
                "operation": operation
            }
        });

        let response = client
            .post(&format!("{}/task/create", base_url))
            .json(&payload)
            .send()
            .await?;

        let result: serde_json::Value = response.json().await?;
        println!("Created task: {} (priority {})", result["id"], priority);
    }

    println!();
    Ok(())
}

/// Get system statistics
async fn get_system_stats(client: &Client, base_url: &str) -> Result<(), Box<dyn Error>> {
    let response = client
        .get(&format!("{}/stats", base_url))
        .send()
        .await?;

    if response.status().is_success() {
        let stats: serde_json::Value = response.json().await?;
        
        println!("System Statistics:");
        println!("  Total Workers: {}", stats["total_workers"]);
        println!("  Tasks Processed: {}", stats["total_tasks_processed"]);
        println!("  Tasks Completed: {}", stats["total_tasks_completed"]);
        println!("  Tasks Failed: {}", stats["total_tasks_failed"]);
        println!("  Uptime: {} seconds", stats["uptime_seconds"]);
        
        if let Some(workers) = stats["workers"].as_array() {
            println!("  Worker Details:");
            for worker in workers {
                println!("    Worker {}: {} processed, {} completed (port {})",
                    worker["id"], 
                    worker["tasks_processed"],
                    worker["tasks_completed"],
                    worker["port"]
                );
            }
        }
    }

    Ok(())
}