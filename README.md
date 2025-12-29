# Coffee Quality Management Platform

A comprehensive system for Thai coffee farmers, processors, and roasters to manage quality control, traceability, and operations.

## Features

- 🌱 **Farm Management**: Track plots, varieties, and harvest data
- 📊 **Quality Control**: SCA cupping scores, green bean grading, AI defect detection
- 🔗 **Traceability**: QR codes linking to complete lot history
- 📦 **Inventory**: Multi-stage tracking from cherry to roasted bean
- ☕ **Roast Profiles**: Record and replicate roasting profiles
- 🌤️ **Weather Integration**: Forecasts and harvest recommendations
- 📜 **Certifications**: Track Thai GAP, Organic, Fair Trade, etc.
- 📱 **Mobile-First**: PWA with offline support
- 🇹🇭 **Thai Language**: Native Thai interface with English support
- 💬 **LINE Integration**: Notifications via LINE messaging

## Tech Stack

- **Backend**: Rust with Axum web framework
- **Database**: PostgreSQL with SQLx
- **Frontend**: React PWA (planned)
- **WASM**: Rust WebAssembly for client-side computation
- **AI Service**: AWS Lambda + SageMaker for defect detection

## Project Structure

```
coffee-quality-management/
├── backend/          # Rust backend API server
├── shared/           # Shared types and models
├── wasm/             # WebAssembly modules
├── config/           # Configuration files
└── migrations/       # Database migrations (to be added)
```

## Getting Started

### Prerequisites

- Rust 1.75+
- PostgreSQL 15+
- SQLx CLI (`cargo install sqlx-cli`)

### Development Setup

1. Clone the repository
2. Copy `.env.example` to `.env` and configure
3. Create the database:
   ```bash
   createdb coffee_qm_dev
   ```
4. Run migrations:
   ```bash
   sqlx migrate run --source backend/migrations
   ```
5. Start the server:
   ```bash
   cargo run --bin cqm-server
   ```

### Environment Variables

All configuration can be set via environment variables with the `CQM__` prefix:

- `CQM_ENVIRONMENT`: development or production
- `CQM__DATABASE__URL`: PostgreSQL connection string
- `CQM__JWT__SECRET`: Secret key for JWT tokens
- See `.env.example` for full list

## API Documentation

API documentation will be available at `/api/docs` when the server is running.

## License

MIT
