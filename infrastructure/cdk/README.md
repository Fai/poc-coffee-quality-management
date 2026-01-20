# CDK Infrastructure

## Why CDK over Terraform?

| Feature | CDK | Terraform |
|---------|-----|-----------|
| AWS Native | ✅ First-party | Third-party provider |
| State Management | CloudFormation (automatic) | S3+DynamoDB (manual) |
| Language | TypeScript (same as frontend) | HCL (new syntax) |
| Type Safety | Full TypeScript types | Limited |
| Drift Detection | CloudFormation native | Manual |
| Cost Tracking | Native tag propagation | Manual |
| Rollback | Automatic on failure | Manual |

## Quick Start

```bash
cd infrastructure/cdk

# Install dependencies
npm install

# Bootstrap CDK (one-time per account/region)
npx cdk bootstrap

# Deploy test environment
npm run deploy:test

# View diff before deploy
npm run diff

# Destroy (stops all costs)
npm run destroy:test
```

## Cost Tracking

All resources automatically tagged with:
- `Project: CoffeeQualityManagement`
- `Environment: test`
- `CostCenter: CC-COFFEE-001`
- `ManagedBy: cdk`

### View Costs

```bash
# Via npm script
npm run costs

# Or AWS Console
# Cost Explorer → Filter by Tag → Project = CoffeeQualityManagement
```

### Budget Alert

Automatically creates AWS Budget:
- **Limit**: $50/month (configurable)
- **Alert**: At 80% of budget

## Stack Resources

| Resource | Type | Cost/month |
|----------|------|------------|
| VPC | No NAT Gateway | $0 |
| RDS | db.t3.micro | $12 |
| ECS | Fargate Spot | $5 |
| ALB | Application LB | $16 |
| Lambda | Pay-per-request | $3 |
| S3 | Standard | $1 |
| **Total** | | **~$37** |

## Configuration

Edit `bin/app.ts` to change:

```typescript
new CoffeeQMStack(app, 'CoffeeQM-test', {
  environment: 'test',
  costCenter: 'CC-COFFEE-001',
  monthlyBudget: 50,        // Budget alert threshold
  useFargateSpot: true,     // 70% cheaper compute
  dbInstanceClass: 'db.t3.micro',  // Smallest RDS
});
```

## Version History

CloudFormation tracks all changes:

```bash
# View stack events
aws cloudformation describe-stack-events --stack-name CoffeeQM-test

# View change sets before deploy
npx cdk diff
```
