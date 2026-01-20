#!/bin/bash
# Enable cost allocation tags and create budget for this project
set -e

PROJECT_NAME="CoffeeQualityManagement"
AWS_REGION="${AWS_REGION:-ap-southeast-1}"
MONTHLY_BUDGET="${1:-50}"
EMAIL="${2:-}"

echo "Setting up cost tracking for ${PROJECT_NAME}..."

# Get account ID
ACCOUNT_ID=$(aws sts get-caller-identity --query Account --output text)

# 1. Activate cost allocation tags (requires Cost Explorer enabled)
echo "Note: Activate these tags in AWS Billing Console > Cost Allocation Tags:"
echo "  - Project"
echo "  - Environment" 
echo "  - CostCenter"
echo ""

# 2. Create budget with alert
if [ -n "$EMAIL" ]; then
    echo "Creating budget alert at \$${MONTHLY_BUDGET}/month..."
    
    cat > /tmp/budget.json << EOF
{
    "BudgetName": "${PROJECT_NAME}-monthly-budget",
    "BudgetLimit": {
        "Amount": "${MONTHLY_BUDGET}",
        "Unit": "USD"
    },
    "BudgetType": "COST",
    "TimeUnit": "MONTHLY",
    "CostFilters": {
        "TagKeyValue": [
            "user:Project\$${PROJECT_NAME}"
        ]
    }
}
EOF

    cat > /tmp/notifications.json << EOF
[
    {
        "Notification": {
            "NotificationType": "ACTUAL",
            "ComparisonOperator": "GREATER_THAN",
            "Threshold": 80,
            "ThresholdType": "PERCENTAGE"
        },
        "Subscribers": [
            {
                "SubscriptionType": "EMAIL",
                "Address": "${EMAIL}"
            }
        ]
    },
    {
        "Notification": {
            "NotificationType": "FORECASTED",
            "ComparisonOperator": "GREATER_THAN",
            "Threshold": 100,
            "ThresholdType": "PERCENTAGE"
        },
        "Subscribers": [
            {
                "SubscriptionType": "EMAIL",
                "Address": "${EMAIL}"
            }
        ]
    }
]
EOF

    aws budgets create-budget \
        --account-id "$ACCOUNT_ID" \
        --budget file:///tmp/budget.json \
        --notifications-with-subscribers file:///tmp/notifications.json \
        2>/dev/null || echo "Budget already exists or update needed"
    
    rm /tmp/budget.json /tmp/notifications.json
    echo "Budget created! Alerts at 80% actual and 100% forecasted."
fi

echo ""
echo "=========================================="
echo "Cost Tracking Setup Complete"
echo "=========================================="
echo ""
echo "View costs by project:"
echo "  aws ce get-cost-and-usage \\"
echo "    --time-period Start=$(date -v-30d +%Y-%m-%d),End=$(date +%Y-%m-%d) \\"
echo "    --granularity MONTHLY \\"
echo "    --metrics UnblendedCost \\"
echo "    --group-by Type=TAG,Key=Project \\"
echo "    --filter '{\"Tags\":{\"Key\":\"Project\",\"Values\":[\"${PROJECT_NAME}\"]}}'"
echo ""
echo "Or use: ./scripts/show-costs.sh"
