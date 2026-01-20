# Cost Optimization Guide
## Coffee Quality Management Platform

### Cost Comparison

| Component | Original Test | Optimized Test | Savings |
|-----------|--------------|----------------|---------|
| NAT Gateway | $32 | $0 | 100% |
| RDS Aurora Serverless | $45 | $12 (t3.micro) | 73% |
| ElastiCache Redis | $12 | $0 (in-memory) | 100% |
| SageMaker (always-on) | $50 | $3 (Lambda pay-per-use) | 94% |
| ECS Fargate | $15 | $5 (Spot) | 67% |
| ALB | $16 | $16 | 0% |
| S3 + CloudWatch | $8 | $3 | 63% |
| **Total** | **~$137/mo** | **~$39/mo** | **72%** |

---

### Optimizations Applied

#### 1. No NAT Gateway (-$32/month)
- Test environment uses public subnets only
- ECS tasks get public IPs directly
- Database in private subnet accessed via security groups

#### 2. RDS t3.micro instead of Aurora (-$33/month)
- Free tier eligible for first 12 months
- 20GB storage sufficient for testing
- Single-AZ (no Multi-AZ redundancy needed for test)

#### 3. No ElastiCache (-$12/month)
- Backend handles sessions in-memory
- For test, caching not critical
- Add Redis only when needed for production

#### 4. Lambda + API Gateway for AI (-$47/month vs SageMaker always-on)
- Pay only per request (~$0.0002/request)
- No idle costs when not testing
- Lambda handles image upload + inference
- Can connect to SageMaker Serverless when model is ready
- Estimated: $3-10/month for test workloads

#### 5. Fargate Spot (-$10/month)
- 70% cheaper than on-demand
- Acceptable for test workloads
- May have occasional interruptions

#### 6. Minimal Resources
- 256 CPU / 512MB memory (smallest Fargate)
- 7-day log retention (vs 30 days)
- Keep only 3 Docker images in ECR

---

### Environment Cost Tiers

```
┌─────────────────────────────────────────────────────────────┐
│                    COST BY ENVIRONMENT                       │
├─────────────────────────────────────────────────────────────┤
│                                                              │
│  DEV/TEST     $35-45/mo   █████░░░░░░░░░░░░░░░░░░░░░░░░░░   │
│  (Optimized)              Fargate Spot, t3.micro, Lambda AI │
│                                                              │
│  STAGING      $150-200/mo ████████████░░░░░░░░░░░░░░░░░░░   │
│                           Aurora Serverless, Redis, SM Srvls│
│                                                              │
│  PRODUCTION   $400-600/mo ████████████████████████░░░░░░░   │
│                           Multi-AZ, SageMaker real-time     │
│                                                              │
└─────────────────────────────────────────────────────────────┘
```

---

### Deployment Commands

```bash
# Deploy OPTIMIZED test environment (~$36/month)
aws cloudformation deploy \
  --template-file infrastructure/cloudformation/test-environment-optimized.yaml \
  --stack-name CoffeeQM-test \
  --parameter-overrides Environment=test DatabasePassword=YourPassword123 \
  --capabilities CAPABILITY_IAM

# Deploy FULL test environment (~$137/month) - when needed
aws cloudformation deploy \
  --template-file infrastructure/cloudformation/test-environment.yaml \
  --stack-name CoffeeQM-test-full \
  --parameter-overrides Environment=test DatabasePassword=YourPassword123 \
  --capabilities CAPABILITY_NAMED_IAM
```

---

### When to Upgrade

| Trigger | Action |
|---------|--------|
| Need Redis caching | Add ElastiCache ($12/mo) |
| Test AI features | Deploy SageMaker endpoint ($50/mo) |
| Load testing | Switch to on-demand Fargate |
| Pre-production | Use full `test-environment.yaml` |

---

### Additional Cost Savings Tips

1. **Stop when not in use**
   ```bash
   # Scale to 0 tasks (stops compute costs)
   aws ecs update-service --cluster CoffeeQM-test --service backend --desired-count 0
   
   # Scale back up
   aws ecs update-service --cluster CoffeeQM-test --service backend --desired-count 1
   ```

2. **Use AWS Free Tier**
   - RDS t3.micro: 750 hours/month free (first year)
   - S3: 5GB free
   - CloudWatch: 10 metrics free

3. **Set Budget Alerts**
   ```bash
   aws budgets create-budget \
     --account-id $(aws sts get-caller-identity --query Account --output text) \
     --budget file://budget.json
   ```

4. **Clean up unused resources**
   ```bash
   # Delete stack when done testing
   aws cloudformation delete-stack --stack-name CoffeeQM-test
   ```

---

### Production Cost Optimization

For production, consider:

1. **Savings Plans** - 30-40% savings with 1-year commitment
2. **Reserved Instances** - For RDS if predictable usage
3. **Aurora Serverless v2** - Scales to 0.5 ACU when idle
4. **SageMaker Serverless** - Pay per inference, not per hour
5. **CloudFront** - Cache API responses to reduce backend load
