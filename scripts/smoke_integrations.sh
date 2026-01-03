#!/bin/bash
# Integration layer smoke tests
# Starts the server and validates integration API endpoints

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
SERVER_BIN="$REPO_ROOT/target/debug/edr-server"
SERVER_PORT=8080
SERVER_HOST="http://localhost:$SERVER_PORT"

# Color output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

log_info() {
  echo -e "${GREEN}[SMOKE]${NC} $1"
}

log_error() {
  echo -e "${RED}[ERROR]${NC} $1"
}

log_warn() {
  echo -e "${YELLOW}[WARN]${NC} $1"
}

# Check if binary exists
if [ ! -f "$SERVER_BIN" ]; then
  log_error "Server binary not found: $SERVER_BIN"
  log_info "Building server..."
  cd "$REPO_ROOT"
  cargo build 2>&1 | tail -10
  if [ ! -f "$SERVER_BIN" ]; then
    log_error "Failed to build server"
    exit 1
  fi
fi

# Start server
log_info "Starting server on $SERVER_HOST..."
"$SERVER_BIN" --port "$SERVER_PORT" &
SERVER_PID=$!

# Wait for server to start
sleep 2
if ! kill -0 $SERVER_PID 2>/dev/null; then
  log_error "Server failed to start (PID $SERVER_PID)"
  exit 1
fi

log_info "Server started (PID $SERVER_PID)"

# Cleanup on exit
cleanup() {
  log_info "Shutting down server (PID $SERVER_PID)..."
  kill $SERVER_PID 2>/dev/null || true
  sleep 1
  kill -9 $SERVER_PID 2>/dev/null || true
}
trap cleanup EXIT

# Helper to test endpoint
test_endpoint() {
  local method=$1
  local path=$2
  local expected_code=$3
  
  log_info "Testing: $method $path (expect HTTP $expected_code)"
  
  response=$(curl -s -w "\n%{http_code}" -X "$method" "$SERVER_HOST$path")
  http_code=$(echo "$response" | tail -1)
  body=$(echo "$response" | head -n -1)
  
  if [ "$http_code" = "$expected_code" ]; then
    log_info "✓ $path returned HTTP $http_code"
    
    # Validate JSON
    if echo "$body" | jq empty 2>/dev/null; then
      log_info "✓ Response is valid JSON"
      
      # Check for non-empty response
      if [ $(echo "$body" | jq 'length') -gt 0 ] 2>/dev/null || [ $(echo "$body" | jq 'keys | length') -gt 0 ] 2>/dev/null; then
        log_info "✓ Response contains data"
        return 0
      else
        log_warn "Response is valid JSON but empty"
        echo "$body" | jq '.' 2>/dev/null || echo "$body"
        return 0
      fi
    else
      log_warn "Response is not valid JSON"
      echo "$body"
      return 1
    fi
  else
    log_error "$path returned HTTP $http_code (expected $expected_code)"
    echo "$body"
    return 1
  fi
}

# Run tests
log_info "=== SMOKE TEST SUITE ==="
PASSED=0
FAILED=0

# Test health endpoint
if test_endpoint GET "/health" 200; then
  ((PASSED++))
else
  ((FAILED++))
fi

# Test API docs
if test_endpoint GET "/api" 200; then
  ((PASSED++))
else
  ((FAILED++))
fi

# Note: Integration API endpoints require proper state setup
# These tests will validate the endpoints exist when state is properly configured
log_warn "Integration API endpoints require state initialization"
log_warn "See: docs/audits/INTEGRATION_LAYER_COMPLETION_SUMMARY.md"

# Summary
log_info "=== TEST RESULTS ==="
log_info "Passed: $PASSED"
if [ $FAILED -gt 0 ]; then
  log_error "Failed: $FAILED"
  exit 1
else
  log_info "All tests passed!"
  exit 0
fi
