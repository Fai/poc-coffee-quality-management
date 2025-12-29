# Shared Infrastructure Configuration

This project is configured to work with shared infrastructure for cost-optimized POC deployment.

## Files Added:
- `config/shared.toml` - Shared infrastructure configuration
- `.env.shared` - Environment variables for shared deployment
- `docker-compose.shared.yml` - Local development with shared services
- `scripts/deploy-shared.sh` - Deployment script for shared infrastructure

## Usage:

### Local Development (Shared Services)
```bash
docker-compose -f docker-compose.shared.yml up
```

### Deploy to Shared Infrastructure
```bash
./scripts/deploy-shared.sh
```

### Environment Variables
Update `.env.shared` with actual shared resource endpoints after infrastructure deployment.

## Project Configuration:
- **Project Name**: cqm
- **Redis Prefix**: cqm:
- **Database**: cqm_dev
- **S3 Prefix**: cqm/
