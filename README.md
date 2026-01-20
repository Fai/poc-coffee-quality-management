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
- 📈 **Reporting**: Harvest yield, quality trends, processing efficiency
- 🔄 **Offline Sync**: Work offline with automatic sync when connected

## Tech Stack

- **Backend**: Rust with Axum web framework
- **Database**: PostgreSQL with SQLx
- **Frontend**: React PWA with Vite, TypeScript, Tailwind CSS
- **State**: Zustand + React Query
- **i18n**: i18next (Thai/English)
- **WASM**: Rust WebAssembly for client-side computation
- **AI Service**: AWS Lambda + SageMaker for defect detection

## Project Structure

```
coffee-quality-management/
├── backend/              # Rust backend API server
│   ├── src/
│   │   ├── handlers/     # HTTP request handlers
│   │   ├── services/     # Business logic
│   │   ├── middleware/   # Auth, logging
│   │   └── routes/       # API route definitions
│   └── migrations/       # Database migrations
├── frontend/             # React PWA
│   └── src/
│       ├── components/   # Reusable UI components
│       ├── pages/        # Page components
│       ├── services/     # API client, auth store
│       └── i18n/         # Translations (th, en)
├── shared/               # Shared types and models
├── wasm/                 # WebAssembly modules
├── ai-defect-detection/  # AWS Lambda + SageMaker
└── config/               # Configuration files
```

## Getting Started

### Prerequisites

- Rust 1.75+
- PostgreSQL 15+
- Node.js 20+
- SQLx CLI (`cargo install sqlx-cli`)

### Development Setup

1. Clone the repository

2. Set up the database:
   ```bash
   # Start PostgreSQL (using Docker)
   docker-compose -f docker-compose.dev.yml up -d
   
   # Or create database manually
   createdb coffee_qm_dev
   ```

3. Configure environment:
   ```bash
   cp .env.example .env
   # Edit .env with your settings
   ```

4. Run migrations:
   ```bash
   sqlx migrate run --source backend/migrations
   ```

5. Start the backend:
   ```bash
   cargo run --bin cqm-server
   ```

6. Start the frontend:
   ```bash
   cd frontend
   npm install
   npm run dev
   ```

### Environment Variables

All configuration can be set via environment variables with the `CQM__` prefix:

- `CQM_ENVIRONMENT`: development or production
- `CQM__DATABASE__URL`: PostgreSQL connection string
- `CQM__JWT__SECRET`: Secret key for JWT tokens
- See `.env.example` for full list

## API Endpoints

### Authentication
- `POST /api/auth/register` - Register business
- `POST /api/auth/login` - Login
- `POST /api/auth/refresh` - Refresh token

### Core Resources
- `/api/plots` - Plot management
- `/api/lots` - Lot management
- `/api/harvests` - Harvest records
- `/api/processing` - Processing records
- `/api/gradings` - Green bean grading
- `/api/cupping` - Cupping sessions
- `/api/inventory` - Inventory transactions
- `/api/roasting` - Roast sessions

### Reports
- `GET /api/reports/dashboard` - Dashboard metrics
- `GET /api/reports/harvest-yield` - Harvest yield report
- `GET /api/reports/quality-trend` - Quality trend report
- `GET /api/reports/processing-efficiency` - Processing efficiency

### Sync (Offline Support)
- `POST /api/sync/changes` - Get changes since last sync
- `POST /api/sync/apply` - Apply pending changes
- `GET /api/sync/conflicts` - Get pending conflicts
- `POST /api/sync/conflicts/resolve` - Resolve conflict

### Public
- `GET /api/trace/:code` - Public traceability view (QR code landing)

## License

MIT
