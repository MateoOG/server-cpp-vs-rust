# #!/bin/bash

# # Function to wait until a port is open
# wait_for_port() {
#     local port=$1
#     echo "Waiting for port $port:"
#     while ! nc -z localhost "$port" >/dev/null 2>&1; do
#         sleep 1
#     done
#     echo "Port $port is ready!"
# }

# echo "🔨 Building C++ system:"
# cd task-processing-system-cpp
# make || { echo " C++ build failed"; exit 1; }
# cd ..

# echo "🔨 Building Rust system:"
# cd task-processing-system-rs
# cargo build || { echo " Rust build failed"; exit 1; }
# cd ..

# echo "Generating C++ system documentation:"
# cd task-processing-system-cpp
# make docs || { echo " C++ docs failed"; exit 1; } #//FIXME add path where they are.
# cd ..

# echo "Generating Rust system documentation:"
# cd task-processing-system-rs
# cargo doc || { echo " Rust docs failed"; exit 1; } #//FIXME add path where they are.
# cd ..

# Open each server in its own terminal
echo "Starting C++ server in new terminal:"
gnome-terminal -- bash -c "cd task-processing-system-cpp && make run-config; exec bash" &

echo "Starting Rust server in new terminal:"
gnome-terminal -- bash -c "cd task-processing-system-rs && RUST_LOG=debug cargo run --config config.toml; exec bash" &

# Adjust ports
CPP_PORT=5000
RUST_PORT=7000

wait_for_port $CPP_PORT
wait_for_port $RUST_PORT

echo "Running performance test: C++ vs Rust"
python3 perf_test_cpp_vs_rust.py

echo "Done! Servers are still running in their own terminals."
