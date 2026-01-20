#!/bin/bash
# E2E Test Runner for Coffee Quality Management Platform
set -e

PROJECT_NAME="CoffeeQualityManagement"
ENVIRONMENT="${1:-test}"
AWS_REGION="${AWS_REGION:-ap-southeast-1}"
STACK_NAME="${PROJECT_NAME}-${ENVIRONMENT}"

echo "=========================================="
echo "Running E2E Tests - ${ENVIRONMENT}"
echo "=========================================="

# Get API endpoint
ALB_DNS=$(aws cloudformation describe-stacks \
    --stack-name "$STACK_NAME" \
    --query "Stacks[0].Outputs[?OutputKey=='ALBDnsName'].OutputValue" \
    --output text \
    --region "$AWS_REGION")

API_URL="http://${ALB_DNS}/api"

echo "API URL: ${API_URL}"

# Wait for service to be healthy
echo "Waiting for service to be healthy..."
for i in {1..30}; do
    if curl -sf "${API_URL}/health" > /dev/null; then
        echo "Service is healthy!"
        break
    fi
    echo "Waiting... ($i/30)"
    sleep 10
done

# Run health check
echo ""
echo "=== Health Check ==="
curl -s "${API_URL}/health" | jq .

# Test registration
echo ""
echo "=== Test Registration ==="
REGISTER_RESPONSE=$(curl -s -X POST "${API_URL}/auth/register" \
    -H "Content-Type: application/json" \
    -d '{
        "business_name": "Test Coffee Farm",
        "business_type": "farmer",
        "business_code": "TEST",
        "owner_name": "Test User",
        "email": "test@example.com",
        "password": "testpassword123",
        "preferred_language": "th"
    }')

echo "$REGISTER_RESPONSE" | jq .

# Extract token
ACCESS_TOKEN=$(echo "$REGISTER_RESPONSE" | jq -r '.access_token // empty')

if [ -z "$ACCESS_TOKEN" ]; then
    echo "Registration failed or user already exists. Trying login..."
    
    LOGIN_RESPONSE=$(curl -s -X POST "${API_URL}/auth/login" \
        -H "Content-Type: application/json" \
        -d '{
            "email": "test@example.com",
            "password": "testpassword123"
        }')
    
    ACCESS_TOKEN=$(echo "$LOGIN_RESPONSE" | jq -r '.access_token')
    echo "Login response:"
    echo "$LOGIN_RESPONSE" | jq .
fi

if [ -z "$ACCESS_TOKEN" ] || [ "$ACCESS_TOKEN" == "null" ]; then
    echo "ERROR: Could not obtain access token"
    exit 1
fi

echo "Access token obtained successfully"

# Test authenticated endpoints
echo ""
echo "=== Test Dashboard ==="
curl -s "${API_URL}/reports/dashboard" \
    -H "Authorization: Bearer ${ACCESS_TOKEN}" | jq .

echo ""
echo "=== Test Create Plot ==="
PLOT_RESPONSE=$(curl -s -X POST "${API_URL}/plots" \
    -H "Authorization: Bearer ${ACCESS_TOKEN}" \
    -H "Content-Type: application/json" \
    -d '{
        "name": "Test Plot A",
        "area_rai": 5.5,
        "altitude_meters": 1200,
        "shade_coverage_percent": 40,
        "varieties": [{"variety": "Typica", "planting_date": "2020-01-15"}]
    }')
echo "$PLOT_RESPONSE" | jq .

PLOT_ID=$(echo "$PLOT_RESPONSE" | jq -r '.id // empty')

echo ""
echo "=== Test List Plots ==="
curl -s "${API_URL}/plots" \
    -H "Authorization: Bearer ${ACCESS_TOKEN}" | jq .

echo ""
echo "=== Test Create Lot ==="
LOT_RESPONSE=$(curl -s -X POST "${API_URL}/lots" \
    -H "Authorization: Bearer ${ACCESS_TOKEN}" \
    -H "Content-Type: application/json" \
    -d '{
        "name": "Test Lot 001",
        "stage": "cherry"
    }')
echo "$LOT_RESPONSE" | jq .

echo ""
echo "=== Test List Lots ==="
curl -s "${API_URL}/lots" \
    -H "Authorization: Bearer ${ACCESS_TOKEN}" | jq .

echo ""
echo "=== Test Sync Endpoint ==="
curl -s -X POST "${API_URL}/sync/changes" \
    -H "Authorization: Bearer ${ACCESS_TOKEN}" \
    -H "Content-Type: application/json" \
    -d '{
        "since_version": 0,
        "device_id": "test-device-001"
    }' | jq .

echo ""
echo "=========================================="
echo "E2E Tests Complete!"
echo "=========================================="
