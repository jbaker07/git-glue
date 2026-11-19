#!/bin/bash
# smoke_stack.sh - Verify EDR stack is running correctly
# Works on macOS and Linux
#
# Usage: ./scripts/smoke_stack.sh [--verbose]

set -e

VERBOSE=false
BASE_URL="${EDR_API_URL:-http://localhost:3000}"
FAILED=0

for arg in "$@"; do
    case $arg in
        --verbose|-v) VERBOSE=true ;;
    esac
done

log() {
    if [ "$VERBOSE" = true ]; then
        echo "$@"
    fi
}

check_endpoint() {
    local name="$1"
    local url="$2"
    local expected_field="$3"
    
    log "Checking $name at $url..."
    
    # Get response body and status code separately (macOS compatible)
    local http_code
    local body
    body=$(curl -s -o /dev/stdout -w '%{http_code}' "$url" 2>/dev/null)
    http_code="${body: -3}"
    body="${body:0:${#body}-3}"
    
    if [ "$http_code" != "200" ]; then
        echo "✗ $name - HTTP $http_code"
        FAILED=$((FAILED + 1))
        return 1
    fi
    
    # Check for valid JSON
    if ! echo "$body" | jq empty 2>/dev/null; then
        echo "✗ $name - Invalid JSON response"
        FAILED=$((FAILED + 1))
        return 1
    fi
    
    # Check for expected field if specified
    if [ -n "$expected_field" ]; then
        if ! echo "$body" | jq -e ".$expected_field" >/dev/null 2>&1; then
            echo "✗ $name - Missing field: $expected_field"
            log "Response: $body"
            FAILED=$((FAILED + 1))
            return 1
        fi
    fi
    
    echo "✓ $name - OK"
    log "Response: $(echo "$body" | jq -c '.' | head -c 200)..."
    return 0
}

echo "=========================================="
echo " EDR Stack Smoke Test"
echo "=========================================="
echo "Base URL: $BASE_URL"
echo ""

# Check health endpoint
check_endpoint "Health" "$BASE_URL/api/health" "status" || true

# Check capabilities endpoint
check_endpoint "Capabilities" "$BASE_URL/api/capabilities" "sources" || true

# Check integrations endpoint
check_endpoint "Integrations" "$BASE_URL/api/integrations" "" || true

# Check signals endpoint (core telemetry)
check_endpoint "Signals" "$BASE_URL/api/signals" "" || true

echo ""
echo "=========================================="
if [ $FAILED -eq 0 ]; then
    echo " All checks passed ✓"
    echo "=========================================="
    exit 0
else
    echo " $FAILED check(s) failed ✗"
    echo "=========================================="
    exit 1
fi
