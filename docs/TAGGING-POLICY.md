# AWS Resource Tagging Policy
## Coffee Quality Management Platform

### Purpose

This document defines the mandatory and optional tags for all AWS resources in the Coffee Quality Management Platform. Consistent tagging enables:

- Cost allocation and tracking
- Resource organization
- Automation and governance
- Security and compliance

---

## Required Tags

All AWS resources MUST have these tags:

| Tag Key | Description | Allowed Values | Example |
|---------|-------------|----------------|---------|
| `Project` | Project identifier | `CoffeeQualityManagement` | `CoffeeQualityManagement` |
| `Environment` | Deployment environment | `dev`, `test`, `staging`, `prod` | `test` |
| `Owner` | Responsible team | Team name | `coffee-platform-team` |
| `CostCenter` | Cost allocation code | `CC-COFFEE-XXX` | `CC-COFFEE-001` |

---

## Optional Tags

These tags are recommended for additional context:

| Tag Key | Description | Allowed Values | Example |
|---------|-------------|----------------|---------|
| `Component` | System component | `backend`, `frontend`, `database`, `cache`, `ai`, `storage`, `network` | `backend` |
| `ManagedBy` | IaC tool used | `cloudformation`, `terraform`, `manual` | `cloudformation` |
| `DataClassification` | Data sensitivity | `public`, `internal`, `confidential`, `restricted` | `confidential` |
| `Backup` | Backup policy | `daily`, `weekly`, `monthly`, `none` | `daily` |
| `AutoShutdown` | Auto-shutdown for cost savings | `true`, `false` | `true` |
| `CreatedBy` | Creator identifier | Email or IAM user | `deploy-pipeline` |

---

## Tag Values by Environment

### Development (`dev`)
```yaml
Project: CoffeeQualityManagement
Environment: dev
Owner: coffee-platform-team
CostCenter: CC-COFFEE-001
AutoShutdown: true
```

### Test (`test`)
```yaml
Project: CoffeeQualityManagement
Environment: test
Owner: coffee-platform-team
CostCenter: CC-COFFEE-001
AutoShutdown: true
```

### Staging (`staging`)
```yaml
Project: CoffeeQualityManagement
Environment: staging
Owner: coffee-platform-team
CostCenter: CC-COFFEE-001
AutoShutdown: false
```

### Production (`prod`)
```yaml
Project: CoffeeQualityManagement
Environment: prod
Owner: coffee-platform-team
CostCenter: CC-COFFEE-001
Backup: daily
DataClassification: confidential
```

---

## Component-Specific Tags

### Backend (ECS)
```yaml
Component: backend
ManagedBy: cloudformation
```

### Database (RDS)
```yaml
Component: database
Backup: daily
DataClassification: confidential
```

### Cache (ElastiCache)
```yaml
Component: cache
```

### Storage (S3)
```yaml
Component: storage
DataClassification: internal
```

### AI Service (Lambda/SageMaker)
```yaml
Component: ai
```

---

## AWS Cost Allocation Tags

Enable these tags in AWS Billing Console for cost tracking:

1. Go to AWS Billing Console → Cost Allocation Tags
2. Activate the following user-defined tags:
   - `Project`
   - `Environment`
   - `Owner`
   - `CostCenter`
   - `Component`

---

## Enforcement

### CloudFormation
All CloudFormation templates must include required tags. Example:

```yaml
Tags:
  - Key: Project
    Value: !Ref ProjectName
  - Key: Environment
    Value: !Ref Environment
  - Key: Owner
    Value: !Ref Owner
  - Key: CostCenter
    Value: !Ref CostCenter
```

### AWS Config Rule
Consider implementing `required-tags` AWS Config rule:

```yaml
ConfigRule:
  Type: AWS::Config::ConfigRule
  Properties:
    ConfigRuleName: required-tags
    Source:
      Owner: AWS
      SourceIdentifier: REQUIRED_TAGS
    InputParameters:
      tag1Key: Project
      tag2Key: Environment
      tag3Key: Owner
      tag4Key: CostCenter
```

### Tag Policy (AWS Organizations)
If using AWS Organizations, implement a tag policy:

```json
{
  "tags": {
    "Project": {
      "tag_key": { "@@assign": "Project" },
      "enforced_for": { "@@assign": ["*"] }
    },
    "Environment": {
      "tag_key": { "@@assign": "Environment" },
      "tag_value": { "@@assign": ["dev", "test", "staging", "prod"] },
      "enforced_for": { "@@assign": ["*"] }
    }
  }
}
```

---

## Cost Reporting

### AWS Cost Explorer Filters
Use these filters for cost analysis:

- **By Project**: Filter by `Project` tag
- **By Environment**: Filter by `Environment` tag
- **By Component**: Filter by `Component` tag

### Budget Alerts
Set up AWS Budgets with tag filters:

```yaml
Budget:
  Type: AWS::Budgets::Budget
  Properties:
    Budget:
      BudgetName: CoffeeQM-Test-Monthly
      BudgetLimit:
        Amount: 200
        Unit: USD
      TimeUnit: MONTHLY
      CostFilters:
        TagKeyValue:
          - "user:Project$CoffeeQualityManagement"
          - "user:Environment$test"
```

---

## Compliance Checklist

- [ ] All resources have required tags
- [ ] Cost allocation tags activated in Billing Console
- [ ] AWS Config rule for tag compliance (optional)
- [ ] Monthly cost review by tag
- [ ] Budget alerts configured
