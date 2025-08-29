# Makefile for Task Processing System
# Using cpp-httplib (header-only HTTP library)

# Compiler and flags
CXX = g++
CXXFLAGS = -std=c++17 -Wall -Wextra -O2 -pthread
DEBUG_FLAGS = -std=c++17 -Wall -Wextra -g -pthread -DDEBUG
INCLUDE_DIRS = -I./src -I/usr/local/include -I/usr/include
LIBS = -pthread

# Check for system-specific library paths
UNAME_S := $(shell uname -s)
ifeq ($(UNAME_S),Darwin)
    INCLUDE_DIRS += -I/opt/homebrew/include -I/usr/local/include
endif

# Directories
SRC_DIR = src
BUILD_DIR = build
TEST_DIR = tests
DOCS_DIR = docs

# Source files
SOURCES = $(SRC_DIR)/main.cpp \
          $(SRC_DIR)/task.cpp \
          $(SRC_DIR)/calculations.cpp \
          $(SRC_DIR)/worker.cpp \
          $(SRC_DIR)/orchestrator.cpp

# Object files
OBJECTS = $(SOURCES:$(SRC_DIR)/%.cpp=$(BUILD_DIR)/%.o)

# Test files
TEST_SOURCES = $(wildcard $(TEST_DIR)/*.cpp)
TEST_OBJECTS = $(TEST_SOURCES:$(TEST_DIR)/%.cpp=$(BUILD_DIR)/test_%.o)

# Target executable
TARGET = $(BUILD_DIR)/task_processor

# Test executable
TEST_TARGET = $(BUILD_DIR)/test_runner

# Default target - only compile if dependencies are available
.PHONY: all
all: check-deps directories $(TARGET)

# Create necessary directories
.PHONY: directories
directories:
	@mkdir -p $(BUILD_DIR)
	@mkdir -p $(DOCS_DIR)

# Check if dependencies are available (but don't install automatically)
.PHONY: check-deps
check-deps:
	@if [ ! -f "$(SRC_DIR)/httplib.h" ]; then \
		echo "Error: cpp-httplib header not found at $(SRC_DIR)/httplib.h"; \
		echo "Run 'make install-deps' to install dependencies first"; \
		exit 1; \
	fi
	@echo "Dependencies check passed"

# Install system dependencies (SEPARATED from compilation)
.PHONY: install-deps
install-deps:
	@echo "Installing system dependencies..."
	@./build.sh --install-deps
	@$(MAKE) install-httplib
	@echo "Dependencies installed successfully!"

# Install cpp-httplib header only (separated)
.PHONY: install-httplib
install-httplib:
	@if [ ! -f "$(SRC_DIR)/httplib.h" ]; then \
		echo "Downloading cpp-httplib header..."; \
		if command -v wget >/dev/null 2>&1; then \
			wget -q https://raw.githubusercontent.com/yhirose/cpp-httplib/master/httplib.h -O $(SRC_DIR)/httplib.h; \
		elif command -v curl >/dev/null 2>&1; then \
			curl -s https://raw.githubusercontent.com/yhirose/cpp-httplib/master/httplib.h -o $(SRC_DIR)/httplib.h; \
		else \
			echo "Error: Neither wget nor curl available. Please install one of them."; \
			echo "Or manually download httplib.h to $(SRC_DIR)/"; \
			exit 1; \
		fi; \
		echo "cpp-httplib installed to $(SRC_DIR)/httplib.h"; \
	else \
		echo "cpp-httplib already available at $(SRC_DIR)/httplib.h"; \
	fi

# Production build (incremental compilation - only recompile changed files)
$(TARGET): $(OBJECTS)
	@echo "Linking $@..."
	$(CXX) $(OBJECTS) -o $@ $(LIBS)
	@echo "Build complete: $@"

# Compile source files (incremental - only compile if source is newer than object)
$(BUILD_DIR)/%.o: $(SRC_DIR)/%.cpp
	@echo "Compiling $<..."
	$(CXX) $(CXXFLAGS) $(INCLUDE_DIRS) -c $< -o $@

# Debug build
.PHONY: debug
debug: CXXFLAGS = $(DEBUG_FLAGS)
debug: check-deps directories $(TARGET)
	@echo "Debug build complete"

# Release build with optimizations
.PHONY: release
release: CXXFLAGS += -O3 -DNDEBUG
release: check-deps directories $(TARGET)
	@echo "Release build complete"

# Test build (incremental)
.PHONY: test
test: check-deps directories $(TEST_TARGET)
	@echo "Running unit tests..."
	./$(TEST_TARGET)

# Test compilation (incremental)
$(TEST_TARGET): $(TEST_OBJECTS) $(filter-out $(BUILD_DIR)/main.o, $(OBJECTS))
	@echo "Linking test executable..."
	$(CXX) $^ -o $@ $(LIBS) -lgtest -lgtest_main

# Compile test files (incremental)
$(BUILD_DIR)/test_%.o: $(TEST_DIR)/%.cpp
	@echo "Compiling test $<..."
	$(CXX) $(CXXFLAGS) $(INCLUDE_DIRS) -c $< -o $@

# Clean build artifacts (but keep dependencies)
.PHONY: clean
clean:
	@echo "Cleaning build artifacts..."
	rm -rf $(BUILD_DIR)
	rm -rf $(DOCS_DIR)

# Clean everything including downloaded httplib
.PHONY: clean-all
clean-all: clean
	@echo "Removing downloaded dependencies..."
	rm -f $(SRC_DIR)/httplib.h
	@echo "All artifacts and dependencies cleaned"

# Generate documentation with Doxygen
.PHONY: docs
docs: directories
	@echo "Generating documentation..."
	doxygen Doxyfile || (echo "Creating default Doxyfile..." && \
	echo "PROJECT_NAME = \"Task Processing System\"" > Doxyfile && \
	echo "INPUT = src/" >> Doxyfile && \
	echo "OUTPUT_DIRECTORY = docs/" >> Doxyfile && \
	echo "GENERATE_HTML = YES" >> Doxyfile && \
	echo "GENERATE_LATEX = NO" >> Doxyfile && \
	echo "EXTRACT_ALL = YES" >> Doxyfile && \
	echo "HAVE_DOT = YES" >> Doxyfile && \
	doxygen Doxyfile)
	@echo "Documentation generated in $(DOCS_DIR)/"

# Performance test
.PHONY: perf-test
perf-test: $(TARGET)
	@echo "Running performance tests..."
	python3 tests/performance_test.py

# Unit test (C++)
.PHONY: unit-test
unit-test: test

# Integration test
.PHONY: integration-test
integration-test: $(TARGET)
	@echo "Running integration tests..."
	python3 tests/integration_test.py

# Full test suite
.PHONY: test-all
test-all: unit-test integration-test perf-test

# Run the application with default config (no recompilation if already built)
.PHONY: run
run: $(TARGET)
	@echo "Starting Task Processing System..."
	./$(TARGET)

# Run with custom configuration
.PHONY: run-config
run-config: $(TARGET)
	@echo "Starting with config file..."
	./$(TARGET) -c config.json

# Run with specific parameters
.PHONY: run-custom
run-custom: $(TARGET)
	@echo "Starting with custom parameters..."
	./$(TARGET) --workers 5 --threads 8 --port 9000

# Development setup (install deps + create config)
.PHONY: setup-dev
setup-dev: install-deps
	@echo "Setting up development environment..."
	@echo "Creating sample configuration..."
	@echo '{' > config.json
	@echo '  "num_workers": 3,' >> config.json
	@echo '  "threads_per_worker": 4,' >> config.json
	@echo '  "orchestrator_port": 7000' >> config.json
	@echo '}' >> config.json
	@echo "Development environment ready!"

# Alternative: Use CMake for building (if preferred)
.PHONY: cmake-build
cmake-build: check-deps
	@echo "Building with CMake..."
	mkdir -p build-cmake
	cd build-cmake && cmake .. && make -j4
	@echo "CMake build complete. Executable: build-cmake/task_processor"

# Check code style (requires clang-format)
.PHONY: format
format:
	@echo "Formatting code..."
	find $(SRC_DIR) -name "*.cpp" -o -name "*.h" | xargs clang-format -i
	@echo "Code formatted"

# Static analysis (requires cppcheck)
.PHONY: analyze
analyze:
	@echo "Running static analysis..."
	cppcheck --enable=all --std=c++17 $(SRC_DIR)/ --include=$(SRC_DIR)/

# Memory leak check (requires valgrind)
.PHONY: memcheck
memcheck: debug
	@echo "Running memory leak check..."
	valgrind --leak-check=full --show-leak-kinds=all ./$(TARGET)

# Help target
.PHONY: help
help:
	@echo "Task Processing System Build System (cpp-httplib)"
	@echo ""
	@echo "SETUP COMMANDS:"
	@echo "  install-deps   - Install system dependencies and download httplib"
	@echo "  setup-dev      - Complete development environment setup"
	@echo ""
	@echo "BUILD COMMANDS (require dependencies):"
	@echo "  all            - Build the application (default, incremental)"
	@echo "  debug          - Build with debug symbols"
	@echo "  release        - Build optimized release version"
	@echo "  cmake-build    - Alternative build using CMake"
	@echo ""
	@echo "TEST COMMANDS:"
	@echo "  test           - Run unit tests (incremental build)"
	@echo "  integration-test - Run integration tests"
	@echo "  perf-test      - Run performance tests"
	@echo "  test-all       - Run all tests"
	@echo ""
	@echo "RUN COMMANDS:"
	@echo "  run            - Run with default configuration (no recompile)"
	@echo "  run-config     - Run with config.json"
	@echo "  run-custom     - Run with custom parameters"
	@echo ""
	@echo "OTHER COMMANDS:"
	@echo "  docs           - Generate documentation"
	@echo "  clean          - Remove build artifacts (keep dependencies)"
	@echo "  clean-all      - Remove build artifacts and downloaded dependencies"
	@echo ""
	@echo "WORKFLOW:"
	@echo "  1. make install-deps  (once - install system dependencies)"
	@echo "  2. make              (build - incremental compilation)"
	@echo "  3. make run          (run - no recompilation needed)"
	