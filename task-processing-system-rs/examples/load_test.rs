use reqwest::Client;
use serde_json::json;
use std::error::Error;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Semaphore;
use tokio::time::sleep;

/// Load testing example for the Task Processing System
#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    println!("=== Task Processing System Load Test ===\n");

    let client = Client::new();
    let base_url = "http://localhost:7000";

    // Check if system is healthy
    if !check_health(&client, base_url).await {
        eprintln!("System is not healthy. Make sure it's running with: cargo run");
        return Ok(());
    }

    println!("System is healthy. Starting load tests...\n");

    // Test 1: Sequential task creation
    println!("Test 1: Sequential Task Creation");
    sequential_load_test(&client, base_url, 20).await?;

    // Test 2: Concurrent task creation
    println!("Test 2: Concurrent Task Creation");
    concurrent_load_test(&client, base_url, 50, 10).await?;

    // Test 3: Priority distribution test
    println!("Test 3: Priority Distribution Test");
    priority_load_test(&client, base_url, 30).await?;

    // Test 4: Mixed operations test
    println!("Test 4: Mixed Operations Test");
    mixed_operations_test(&client, base_url, 40).await?;

    // Final system statistics
    println!("Final System Statistics:");
    print_system_stats(&client, base_url).await?;

    println!("\n=== Load test completed! ===");
    Ok(())
}

/// Check if system is healthy
async fn check_health(client: &Client, base_url: &str) -> bool {
    match client.get(&format!("{}/health", base_url)).send().await {
        Ok(response) => response.status().is_success(),
        Err(_) => false,
    }
}

/// Sequential load test - create tasks one by one
async fn sequential_load_test(
    client: &Client,
    base_url: &str,
    num_tasks: usize,
) -> Result<(), Box<dyn Error>> {
    let start_time = Instant::now();
    let mut successful_tasks = 0;

    for i in 0..num_tasks {
        let task_id = format!("seq-load-{:03}", i);
        let payload = json!({
            "id": task_id,
            "title": format!("Sequential Load Test {}", i),
            "priority": (i % 3) + 1, // Mix priorities
            "data": {
                "type": "calculation",
                "input": 5 + (i % 10),
                "operation": "factorial"
            }
        });

        match client
            .post(&format!("{}/task/create", base_url))
            .json(&payload)
            .send()
            .await
        {
            Ok(response) => {
                if response.status().is_success() {
                    successful_tasks += 1;
                }
            }
            Err(e) => eprintln!("Failed to create task {}: {}", task_id, e),
        }
    }

    let duration = start_time.elapsed();
    let rate = successful_tasks as f64 / duration.as_secs_f64();

    println!("  Created {} tasks in {:?} ({:.2} tasks/sec)", 
        successful_tasks, duration, rate);
    println!();

    Ok(())
}

/// Concurrent load test - create tasks concurrently
async fn concurrent_load_test(
    client: &Client,
    base_url: &str,
    num_tasks: usize,
    max_concurrent: usize,
) -> Result<(), Box<dyn Error>> {
    let start_time = Instant::now();
    let semaphore = Arc::new(Semaphore::new(max_concurrent));
    let mut handles = Vec::new();

    for i in 0..num_tasks {
        let client = client.clone();
        let base_url = base_url.to_string();
        let semaphore = Arc::clone(&semaphore);

        let handle = tokio::spawn(async move {
            let _permit = semaphore.acquire().await.unwrap();
            
            let task_id = format!("concurrent-load-{:03}", i);
            let payload = json!({
                "id": task_id,
                "title": format!("Concurrent Load Test {}", i),
                "priority": (i % 3) + 1,
                "data": {
                    "type": "calculation",
                    "input": 3 + (i % 8),
                    "operation": match i % 3 {
                        0 => "factorial",
                        1 => "fibonacci",
                        _ => "prime_check"
                    }
                }
            });

            client
                .post(&format!("{}/task/create", base_url))
                .json(&payload)
                .send()
                .await
                .map(|response| response.status().is_success())
                .unwrap_or(false)
        });

        handles.push(handle);
    }

    // Wait for all tasks to complete
    let results = futures::future::join_all(handles).await;
    let successful_tasks = results
        .into_iter()
        .filter_map(|r| r.ok())
        .filter(|&success| success)
        .count();

    let duration = start_time.elapsed();
    let rate = successful_tasks as f64 / duration.as_secs_f64();

    println!("  Created {} tasks concurrently in {:?} ({:.2} tasks/sec)",
        successful_tasks, duration, rate);
    println!("  Max concurrent: {}", max_concurrent);
    println!();

    Ok(())
}

/// Priority distribution test
async fn priority_load_test(
    client: &Client,
    base_url: &str,
    num_tasks: usize,
) -> Result<(), Box<dyn Error>> {
    let start_time = Instant::now();
    let mut priority_counts = [0usize; 3]; // Low, Medium, High

    // Create tasks with different priority distributions
    for i in 0..num_tasks {
        let priority = match i % 6 {
            0 | 1 => 1, // Low priority (33%)
            2 | 3 => 2, // Medium priority (33%) 
            _ => 3,     // High priority (33%)
        };
        
        priority_counts[priority - 1] += 1;

        let task_id = format!("priority-load-{:03}", i);
        let payload = json!({
            "id": task_id,
            "title": format!("Priority Test {} (P{})", i, priority),
            "priority": priority,
            "data": {
                "type": "calculation",
                "input": 4 + (i % 6),
                "operation": "factorial"
            }
        });

        let _ = client
            .post(&format!("{}/task/create", base_url))
            .json(&payload)
            .send()
            .await;
    }

    let duration = start_time.elapsed();

    println!("  Created {} tasks by priority in {:?}:", num_tasks, duration);
    println!("    Low priority (1): {} tasks", priority_counts[0]);
    println!("    Medium priority (2): {} tasks", priority_counts[1]);
    println!("    High priority (3): {} tasks", priority_counts[2]);
    println!();

    Ok(())
}

/// Mixed operations test
async fn mixed_operations_test(
    client: &Client,
    base_url: &str,
    num_tasks: usize,
) -> Result<(), Box<dyn Error>> {
    let start_time = Instant::now();
    let operations = ["factorial", "fibonacci", "prime_check"];
    let mut op_counts = [0usize; 3];

    for i in 0..num_tasks {
        let op_index = i % 3;
        let operation = operations[op_index];
        op_counts[op_index] += 1;

        let input = match operation {
            "factorial" => 3 + (i % 8),
            "fibonacci" => 10 + (i % 20),
            "prime_check" => if i % 2 == 0 { 97 } else { 101 },
            _ => 5,
        };

        let task_id = format!("mixed-op-{:03}", i);
        let payload = json!({
            "id": task_id,
            "title": format!("Mixed Operation {} ({})", i, operation),
            "priority": 2, // All medium priority
            "data": {
                "type": "calculation",
                "input": input,
                "operation": operation
            }
        });

        let _ = client
            .post(&format!("{}/task/create", base_url))
            .json(&payload)
            .send()
            .await;
    }

    let duration = start_time.elapsed();

    println!("  Created {} mixed operation tasks in {:?}:", num_tasks, duration);
    println!("    Factorial: {} tasks", op_counts[0]);
    println!("    Fibonacci: {} tasks", op_counts[1]);
    println!("    Prime check: {} tasks", op_counts[2]);
    println!();

    // Wait for processing
    sleep(Duration::from_millis(1000)).await;

    Ok(())
}

/// Print system statistics
async fn print_system_stats(client: &Client, base_url: &str) -> Result<(), Box<dyn Error>> {
    let response = client
        .get(&format!("{}/stats", base_url))
        .send()
        .await?;

    if response.status().is_success() {
        let stats: serde_json::Value = response.json().await?;
        
        println!("  Total Workers: {}", stats["total_workers"]);
        println!("  Tasks Processed: {}", stats["total_tasks_processed"]);
        println!("  Tasks Completed: {}", stats["total_tasks_completed"]);
        println!("  Tasks Failed: {}", stats["total_tasks_failed"]);
        println!("  System Uptime: {} seconds", stats["uptime_seconds"]);

        if let Some(workers) = stats["workers"].as_array() {
            println!("  Worker Performance:");
            for (i, worker) in workers.iter().enumerate() {
                println!("    Worker {}: {} processed, load: {}", 
                    i,
                    worker["tasks_processed"],
                    worker["current_load"]
                );
            }
        }

        // Calculate processing rate
        let uptime = stats["uptime_seconds"].as_u64().unwrap_or(1);
        let processed = stats["total_tasks_processed"].as_u64().unwrap_or(0);
        let rate = processed as f64 / uptime as f64;
        println!("  Average Processing Rate: {:.2} tasks/sec", rate);
    }

    Ok(())
}