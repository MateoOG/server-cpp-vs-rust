#!/usr/bin/env python3
"""
Performance Test Script for C++ vs Rust Task Processing Systems
Compares both servers running on different ports with configurable thread counts
"""

import asyncio
import aiohttp
import json
import time
import statistics
import argparse
import sys
from typing import Dict, List, Tuple, Any
from dataclasses import dataclass
import matplotlib.pyplot as plt
import pandas as pd
from concurrent.futures import ThreadPoolExecutor
import threading


@dataclass
class TestConfig:
    """Configuration for performance tests"""
    cpp_port: int = 5000
    rust_port: int = 7000
    num_threads: int = 10
    tasks_per_thread: int = 50
    processing_wait_time: float = 0.5  # Time to wait for task processing
    request_timeout: float = 30.0      # Request timeout in seconds
    operations: List[str] = None
    priorities: List[int] = None
    
    def __post_init__(self):
        if self.operations is None:
            self.operations = ['factorial', 'fibonacci', 'prime_check']
        if self.priorities is None:
            self.priorities = [1, 2, 3]


@dataclass
class PerformanceMetrics:
    """Store performance metrics for a server"""
    server_name: str
    total_tasks: int
    successful_tasks: int
    failed_tasks: int
    total_time: float
    avg_response_time: float
    min_response_time: float
    max_response_time: float
    p95_response_time: float
    throughput: float  # tasks per second
    error_rate: float
    response_times: List[float]


class PerformanceTester:
    def __init__(self, config: TestConfig):
        self.config = config
        self.cpp_url = f"http://localhost:{config.cpp_port}"
        self.rust_url = f"http://localhost:{config.rust_port}"
        self.task_counter = 0
        self.results: Dict[str, PerformanceMetrics] = {}
        
    def get_unique_task_id(self, server_name: str) -> str:
        """Generate unique task ID"""
        self.task_counter += 1
        return f"perf-{server_name}-{self.task_counter:05d}"
    
    async def check_server_health(self, session: aiohttp.ClientSession, url: str, server_name: str) -> bool:
        """Check if server is running and responsive"""
        try:
            async with session.get(f"{url}/stats", timeout=aiohttp.ClientTimeout(total=5)) as response:
                if response.status == 200:
                    data = await response.json()
                    print(f"✓ {server_name} server is running (workers: {data.get('total_workers', 'unknown')})")
                    return True
                else:
                    print(f"✗ {server_name} server returned status {response.status}")
                    return False
        except Exception as e:
            print(f"✗ {server_name} server is not reachable: {e}")
            return False
    
    async def create_and_process_task(self, session: aiohttp.ClientSession, 
                                    base_url: str, server_name: str, 
                                    operation: str, input_val: int, priority: int) -> Tuple[bool, float]:
        """Create and process a single task, return (success, response_time)"""
        start_time = time.time()
        task_id = self.get_unique_task_id(server_name)
        
        try:
            # Create task
            payload = {
                "id": task_id,
                "title": f"Performance test {operation}",
                "priority": priority,
                "data": {
                    "type": "calculation",
                    "input": input_val,
                    "operation": operation
                }
            }
            
            create_start = time.time()
            async with session.post(f"{base_url}/task/create", 
                                  json=payload, 
                                  timeout=aiohttp.ClientTimeout(total=self.config.request_timeout)) as response:
                if response.status != 200:
                    return False, time.time() - start_time
            
            # Wait for processing (configurable)
            await asyncio.sleep(self.config.processing_wait_time)
            
            # Check task status (optional - to verify processing)
            async with session.get(f"{base_url}/task/{task_id}", 
                                 timeout=aiohttp.ClientTimeout(total=self.config.request_timeout)) as response:
                if response.status == 200:
                    task_data = await response.json()
                    #print(f"DEBUG {server_name}: status='{task_data.get('status')}', has_result={'result' in task_data}")
                    # Task should be processed by now
                    status = task_data.get('status')
                    has_result = 'result' in task_data
                    
                    # Accept both 'processing' and 'completed' as successful processing
                    if (status in ['processing', 'completed']) and has_result:
                        if status == 'processing':
                            # Complete the task if it's still in processing
                            async with session.post(f"{base_url}/task/{task_id}/complete",
                                                  timeout=aiohttp.ClientTimeout(total=self.config.request_timeout)) as complete_response:
                                success = complete_response.status == 200
                        else:
                            # Already completed, that's success too
                            success = True
                    else:
                        success = False
                else:
                    success = False
            
            return success, time.time() - start_time
            
        except Exception as e:
            return False, time.time() - start_time
    
    async def run_performance_test_for_server(self, base_url: str, server_name: str) -> PerformanceMetrics:
        """Run performance test for a single server"""
        print(f"\n Starting performance test for {server_name} server...")
        print(f"   URL: {base_url}")
        print(f"   Threads: {self.config.num_threads}")
        print(f"   Tasks per thread: {self.config.tasks_per_thread}")
        print(f"   Total tasks: {self.config.num_threads * self.config.tasks_per_thread}")
        
        successful_tasks = 0
        failed_tasks = 0
        response_times = []
        start_time = time.time()
        
        # Create connector with higher limits
        connector = aiohttp.TCPConnector(limit=100, limit_per_host=50)
        timeout = aiohttp.ClientTimeout(total=self.config.request_timeout * 2, connect=20)
        
        async with aiohttp.ClientSession(connector=connector, timeout=timeout) as session:
            # Create semaphore to limit concurrent requests
            semaphore = asyncio.Semaphore(self.config.num_threads)
            
            async def bounded_task(operation: str, input_val: int, priority: int):
                async with semaphore:
                    return await self.create_and_process_task(session, base_url, server_name, 
                                                            operation, input_val, priority)
            
            # Generate tasks
            tasks = []
            task_count = 0
            for thread_id in range(self.config.num_threads):
                for task_in_thread in range(self.config.tasks_per_thread):
                    # Cycle through operations and priorities
                    operation = self.config.operations[task_count % len(self.config.operations)]
                    priority = self.config.priorities[task_count % len(self.config.priorities)]
                    
                    # Vary input values based on operation
                    if operation == 'factorial':
                        input_val = 5 + (task_count % 10)  # 5-14
                    elif operation == 'fibonacci':
                        input_val = 10 + (task_count % 20)  # 10-29
                    else:  # prime_check
                        input_val = 100 + (task_count % 100)  # 100-199
                    
                    tasks.append(bounded_task(operation, input_val, priority))
                    task_count += 1
            
            # Execute all tasks concurrently
            print(f"   Executing {len(tasks)} tasks...")
            results = await asyncio.gather(*tasks, return_exceptions=True)
            
            # Process results
            for result in results:
                if isinstance(result, Exception):
                    failed_tasks += 1
                    response_times.append(30.0)  # timeout value
                else:
                    success, response_time = result
                    if success:
                        successful_tasks += 1
                    else:
                        failed_tasks += 1
                    response_times.append(response_time)
        
        total_time = time.time() - start_time
        total_tasks = successful_tasks + failed_tasks
        
        # Calculate metrics
        if response_times:
            avg_response_time = statistics.mean(response_times)
            min_response_time = min(response_times)
            max_response_time = max(response_times)
            p95_response_time = statistics.quantiles(response_times, n=20)[18]  # 95th percentile
        else:
            avg_response_time = min_response_time = max_response_time = p95_response_time = 0
        
        throughput = successful_tasks / total_time if total_time > 0 else 0
        error_rate = (failed_tasks / total_tasks * 100) if total_tasks > 0 else 0
        
        metrics = PerformanceMetrics(
            server_name=server_name,
            total_tasks=total_tasks,
            successful_tasks=successful_tasks,
            failed_tasks=failed_tasks,
            total_time=total_time,
            avg_response_time=avg_response_time,
            min_response_time=min_response_time,
            max_response_time=max_response_time,
            p95_response_time=p95_response_time,
            throughput=throughput,
            error_rate=error_rate,
            response_times=response_times
        )
        
        print(f" {server_name} test completed:")
        print(f"   Successful: {successful_tasks}/{total_tasks}")
        print(f"   Throughput: {throughput:.2f} tasks/sec")
        print(f"   Avg Response Time: {avg_response_time:.3f}s")
        print(f"   Error Rate: {error_rate:.1f}%")
        
        return metrics
    
    async def run_comparison_test(self) -> Dict[str, PerformanceMetrics]:
        """Run performance test for both servers"""
        print("=" * 60)
        print(" PERFORMANCE COMPARISON TEST")
        print("=" * 60)
        
        # Check server health first
        async with aiohttp.ClientSession() as session:
            cpp_healthy = await self.check_server_health(session, self.cpp_url, "C++")
            rust_healthy = await self.check_server_health(session, self.rust_url, "Rust")
        
        if not cpp_healthy or not rust_healthy:
            print("\n One or both servers are not running. Please start both servers first.")
            print(f"Expected: C++ on port {self.config.cpp_port}, Rust on port {self.config.rust_port}")
            sys.exit(1)
        
        # Run tests for both servers
        cpp_metrics = await self.run_performance_test_for_server(self.cpp_url, "C++")
        rust_metrics = await self.run_performance_test_for_server(self.rust_url, "Rust")
        
        self.results = {
            "cpp": cpp_metrics,
            "rust": rust_metrics
        }
        
        return self.results
    
    def generate_comparison_report(self):
        """Generate detailed comparison report"""
        if not self.results:
            print("No results to report.")
            return
        
        cpp_metrics = self.results["cpp"]
        rust_metrics = self.results["rust"]
        
        print("\n" + "=" * 60)
        print(" PERFORMANCE COMPARISON REPORT")
        print("=" * 60)
        
        # Summary table
        print(f"\n{'Metric':<25} {'C++':<15} {'Rust':<15} {'Winner':<10}")
        print("-" * 70)
        
        metrics_comparison = [
            ("Throughput (tasks/sec)", cpp_metrics.throughput, rust_metrics.throughput, "higher"),
            ("Avg Response Time (s)", cpp_metrics.avg_response_time, rust_metrics.avg_response_time, "lower"),
            ("P95 Response Time (s)", cpp_metrics.p95_response_time, rust_metrics.p95_response_time, "lower"),
            ("Success Rate (%)", (cpp_metrics.successful_tasks/cpp_metrics.total_tasks*100), 
             (rust_metrics.successful_tasks/rust_metrics.total_tasks*100), "higher"),
            ("Error Rate (%)", cpp_metrics.error_rate, rust_metrics.error_rate, "lower"),
        ]
        
        for metric_name, cpp_val, rust_val, better in metrics_comparison:
            if better == "higher":
                winner = "C++" if cpp_val > rust_val else "Rust" if rust_val > cpp_val else "Tie"
            else:  # lower is better
                winner = "C++" if cpp_val < rust_val else "Rust" if rust_val < cpp_val else "Tie"
            
            print(f"{metric_name:<25} {cpp_val:<15.3f} {rust_val:<15.3f} {winner:<10}")
        
        # Performance improvement calculation
        print(f"\n PERFORMANCE IMPROVEMENTS:")
        throughput_improvement = ((rust_metrics.throughput - cpp_metrics.throughput) / cpp_metrics.throughput * 100) if cpp_metrics.throughput > 0 else 0
        response_improvement = ((cpp_metrics.avg_response_time - rust_metrics.avg_response_time) / cpp_metrics.avg_response_time * 100) if cpp_metrics.avg_response_time > 0 else 0
        
        print(f"   Rust throughput vs C++: {throughput_improvement:+.1f}%")
        print(f"   Rust response time vs C++: {response_improvement:+.1f}% (negative = faster)")
        
        # Detailed stats
        print(f"\n DETAILED STATISTICS:")
        for name, metrics in [("C++", cpp_metrics), ("Rust", rust_metrics)]:
            print(f"\n{name} Server:")
            print(f"   Total Tasks: {metrics.total_tasks}")
            print(f"   Successful: {metrics.successful_tasks} ({metrics.successful_tasks/metrics.total_tasks*100:.1f}%)")
            print(f"   Failed: {metrics.failed_tasks} ({metrics.error_rate:.1f}%)")
            print(f"   Total Time: {metrics.total_time:.2f}s")
            print(f"   Throughput: {metrics.throughput:.2f} tasks/sec")
            print(f"   Avg Response Time: {metrics.avg_response_time:.3f}s")
            print(f"   Min Response Time: {metrics.min_response_time:.3f}s")
            print(f"   Max Response Time: {metrics.max_response_time:.3f}s")
            print(f"   P95 Response Time: {metrics.p95_response_time:.3f}s")
    
    def create_performance_charts(self):
        """Create performance comparison charts"""
        if not self.results:
            print("No results to chart.")
            return
        
        cpp_metrics = self.results["cpp"]
        rust_metrics = self.results["rust"]
        
        # Create subplots
        fig, ((ax1, ax2), (ax3, ax4)) = plt.subplots(2, 2, figsize=(15, 10))
        fig.suptitle('C++ vs Rust Performance Comparison', fontsize=16, fontweight='bold')
        
        # Throughput comparison
        servers = ['C++', 'Rust']
        throughputs = [cpp_metrics.throughput, rust_metrics.throughput]
        bars1 = ax1.bar(servers, throughputs, color=['#FF6B6B', '#4ECDC4'])
        ax1.set_title('Throughput (Tasks/Second)')
        ax1.set_ylabel('Tasks/Second')
        for bar, value in zip(bars1, throughputs):
            ax1.text(bar.get_x() + bar.get_width()/2, bar.get_height() + max(throughputs)*0.01,
                    f'{value:.2f}', ha='center', va='bottom', fontweight='bold')
        
        # Response time comparison
        avg_times = [cpp_metrics.avg_response_time, rust_metrics.avg_response_time]
        bars2 = ax2.bar(servers, avg_times, color=['#FF6B6B', '#4ECDC4'])
        ax2.set_title('Average Response Time')
        ax2.set_ylabel('Seconds')
        for bar, value in zip(bars2, avg_times):
            ax2.text(bar.get_x() + bar.get_width()/2, bar.get_height() + max(avg_times)*0.01,
                    f'{value:.3f}s', ha='center', va='bottom', fontweight='bold')
        
        # Success rate comparison
        success_rates = [cpp_metrics.successful_tasks/cpp_metrics.total_tasks*100,
                        rust_metrics.successful_tasks/rust_metrics.total_tasks*100]
        bars3 = ax3.bar(servers, success_rates, color=['#FF6B6B', '#4ECDC4'])
        ax3.set_title('Success Rate')
        ax3.set_ylabel('Percentage (%)')
        ax3.set_ylim(0, 100)
        for bar, value in zip(bars3, success_rates):
            ax3.text(bar.get_x() + bar.get_width()/2, bar.get_height() + 1,
                    f'{value:.1f}%', ha='center', va='bottom', fontweight='bold')
        
        # Response time distribution (histogram)
        ax4.hist(cpp_metrics.response_times, bins=30, alpha=0.7, label='C++', color='#FF6B6B', density=True)
        ax4.hist(rust_metrics.response_times, bins=30, alpha=0.7, label='Rust', color='#4ECDC4', density=True)
        ax4.set_title('Response Time Distribution')
        ax4.set_xlabel('Response Time (seconds)')
        ax4.set_ylabel('Density')
        ax4.legend()
        
        plt.tight_layout()
        
        # Save chart
        chart_filename = 'performance_comparison.png'
        plt.savefig(chart_filename, dpi=300, bbox_inches='tight')
        print(f"\n Performance chart saved as: {chart_filename}")
        
        # Also create a CSV report
        self.create_csv_report()
    
    def create_csv_report(self):
        """Create CSV report with detailed results"""
        if not self.results:
            return
        
        # Create comparison data
        data = []
        for name, metrics in [("C++", self.results["cpp"]), ("Rust", self.results["rust"])]:
            data.append({
                'Server': name,
                'Total_Tasks': metrics.total_tasks,
                'Successful_Tasks': metrics.successful_tasks,
                'Failed_Tasks': metrics.failed_tasks,
                'Total_Time_Seconds': metrics.total_time,
                'Throughput_TasksPerSec': metrics.throughput,
                'Avg_Response_Time_Sec': metrics.avg_response_time,
                'Min_Response_Time_Sec': metrics.min_response_time,
                'Max_Response_Time_Sec': metrics.max_response_time,
                'P95_Response_Time_Sec': metrics.p95_response_time,
                'Success_Rate_Percent': metrics.successful_tasks/metrics.total_tasks*100,
                'Error_Rate_Percent': metrics.error_rate
            })
        
        df = pd.DataFrame(data)
        csv_filename = 'performance_results.csv'
        df.to_csv(csv_filename, index=False)
        print(f" Detailed results saved as: {csv_filename}")


def main():
    parser = argparse.ArgumentParser(description='Performance test for C++ vs Rust task processing systems')
    parser.add_argument('--cpp-port', type=int, default=5000, help='C++ server port (default: 5000)')
    parser.add_argument('--rust-port', type=int, default=7000, help='Rust server port (default: 7000)')
    parser.add_argument('--threads', type=int, default=10, help='Number of concurrent threads (default: 10)')
    parser.add_argument('--tasks-per-thread', type=int, default=50, help='Tasks per thread (default: 50)')
    parser.add_argument('--operations', nargs='+', default=['factorial', 'fibonacci', 'prime_check'],
                       help='Operations to test (default: factorial fibonacci prime_check)')
    parser.add_argument('--priorities', nargs='+', type=int, default=[1, 2, 3],
                       help='Priority levels to test (default: 1 2 3)')
    parser.add_argument('--processing-wait', type=float, default=0.5, 
                       help='Time to wait for task processing (default: 0.5 seconds)')
    parser.add_argument('--request-timeout', type=float, default=30.0,
                       help='Request timeout in seconds (default: 30)')
    parser.add_argument('--no-charts', action='store_true', help='Skip generating charts')
    
    args = parser.parse_args()
    
    # Create test configuration
    config = TestConfig(
        cpp_port=args.cpp_port,
        rust_port=args.rust_port,
        num_threads=args.threads,
        tasks_per_thread=args.tasks_per_thread,
        processing_wait_time=args.processing_wait,
        request_timeout=args.request_timeout,
        operations=args.operations,
        priorities=args.priorities
    )
    
    print(f"Performance Test Configuration:")
    print(f"  C++ Server: http://localhost:{config.cpp_port}")
    print(f"  Rust Server: http://localhost:{config.rust_port}")
    print(f"  Concurrent Threads: {config.num_threads}")
    print(f"  Tasks per Thread: {config.tasks_per_thread}")
    print(f"  Total Tasks per Server: {config.num_threads * config.tasks_per_thread}")
    print(f"  Operations: {', '.join(config.operations)}")
    print(f"  Priorities: {config.priorities}")
    
    # Run performance test
    tester = PerformanceTester(config)
    
    try:
        # Run the async test
        results = asyncio.run(tester.run_comparison_test())
        
        # Generate reports
        tester.generate_comparison_report()
        
        if not args.no_charts:
            try:
                tester.create_performance_charts()
            except ImportError:
                print("\n⚠️  matplotlib/pandas not available for chart generation")
                print("Install with: pip install matplotlib pandas")
        
        print(f"\n Performance comparison completed!")
        print(f"Both servers tested with {config.num_threads} threads x {config.tasks_per_thread} tasks")
        
    except KeyboardInterrupt:
        print("\n  Test interrupted by user")
    except Exception as e:
        print(f"\n Test failed: {e}")
        sys.exit(1)


if __name__ == "__main__":
    main()

#usage: python3 performance_test.py   --processing-wait 2.0   --request-timeout 60   --threads 5   --tasks-per-thread 20