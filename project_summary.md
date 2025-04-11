# Sensor Monitoring System with Rust and SQLite

## Project Overview

This project implements a time-series optimized utility data logging system using Rust and SQLite. The system consists of two main components:

1. **API Server**: An Axum-based web API that provides CRUD operations for sensor management, data logging, and session tracking.
2. **Web Client**: A separate Axum-based web application that provides a user interface for interacting with the API.

The entire system is designed with performance and scalability in mind, optimizing SQLite for time-series data management.

## Architecture

### Database Design

The SQLite database is specifically optimized for time-series data with:

- Proper indexing strategy for efficient time-based queries
- Table design that separates metadata and readings
- WAL mode for better concurrency
- Optimized query patterns for time range filtering
- Connection pooling for concurrent access
- Support for bulk operations

### API Server

The API server provides a RESTful interface with the following endpoints:

- **Sensor Management**: Create, read, update, and delete sensors
- **Data Logging**: Record individual readings or bulk import data
- **Session Management**: Start and stop logging sessions
- **Data Retrieval**: Query readings with comprehensive filtering
- **Visualization Data**: Optimized endpoints for different chart types
- **System Management**: Health checks, maintenance, and data export

### Web Client

The web client provides a user-friendly interface for:

- Viewing and managing sensors
- Starting and stopping data logging sessions
- Recording manual readings
- Visualizing data through charts and dashboards
- Monitoring sensor status

## Implementation Details

### Technology Stack

- **Backend**: Rust with Axum web framework
- **Database**: SQLite with time-series optimizations
- **Client**: Rust with Axum (server-side rendering) + JavaScript for charts
- **Charts**: Chart.js for data visualization
- **CSS Framework**: Bootstrap 5 for responsive design
- **Deployment**: Docker and Docker Compose

### Performance Optimizations

1. **Database Level**:
   - Composite indices for efficient time-series queries
   - WAL journaling mode for better concurrent access
   - Optimized cache settings
   - Prepared statements for query efficiency
   - Bulk insertion capabilities

2. **API Level**:
   - Connection pooling for efficient database access
   - Request batching for bulk operations
   - Query parameter optimization
   - Asynchronous processing with Tokio

3. **Client Level**:
   - Efficient data fetching patterns
   - Minimal UI updates
   - Resource caching

## Project Structure

```
sensor-monitoring-api/            # Main API project
├── Cargo.toml                    # Project dependencies
├── src/
│   ├── main.rs                   # Application entry point
│   ├── db/                       # Database management
│   │   ├── mod.rs
│   │   ├── schema.rs
│   │   └── migrations.rs
│   ├── models/                   # Data models
│   │   ├── mod.rs
│   │   ├── sensor.rs
│   │   ├── reading.rs
│   │   └── session.rs
│   ├── api/                      # API routes and handlers
│   │   ├── mod.rs
│   │   ├── sensors.rs
│   │   ├── readings.rs
│   │   ├── sessions.rs
│   │   └── system.rs
│   └── utils/                    # Utilities
│       ├── mod.rs
│       ├── error.rs
│       └── csv.rs
├── migrations/                   # SQL migrations
│   └── 001_initial_schema.sql
├── examples/                     # Example applications
│   └── client_app/               # Web client
│       ├── Cargo.toml
│       ├── src/
│       │   ├── main.rs
│       │   ├── dashboard.rs
│       │   └── models.rs
│       └── static/
│           └── js/
│               └── charts.js     # Chart visualization
└── tools/                        # Utility tools
    ├── Cargo.toml
    └── load_test.rs              # Performance testing
```

## Getting Started

### Prerequisites

- Rust 1.70.0 or higher
- SQLite 3.35.0 or higher (bundled with the application)
- Docker and Docker Compose (optional)

### Running the API Server

```bash
# Clone the repository
git clone https://github.com/yourusername/sensor-monitoring-api.git
cd sensor-monitoring-api

# Build and run
cargo build --release
./target/release/sensor-monitoring-api
```

### Running the Web Client

```bash
# Navigate to client app directory
cd examples/client_app

# Set API endpoint (optional)
export API_BASE_URL=http://localhost:3000/api

# Build and run
cargo build --release
./target/release/sensor-client-app
```

### Using Docker Compose

```bash
# Start both API and client with Docker Compose
./deploy.sh
```

## Key Features

1. **Time-Series Optimization**: The database is specifically designed for efficient storage and retrieval of time-series data.

2. **Flexible Sensor Types**: Support for different sensor types (temperature, power, flow, etc.) with appropriate metadata.

3. **Logging Sessions**: Ability to start and stop logging sessions with configurable sampling rates.

4. **Real-time Monitoring**: Dashboard with current status and recent readings.

5. **Data Visualization**: Charts for temperature trends, power consumption, and sensor status.

6. **Data Import/Export**: Tools for bulk data operations.

7. **Database Maintenance**: System utilities for optimization and cleanup.

## Performance Considerations

The system is optimized to handle:

- Up to millions of readings with good query performance
- Multiple concurrent users (limited by SQLite's concurrency model)
- Real-time data logging at reasonable rates (hundreds of readings per second)

For significantly larger deployments or higher throughput requirements, consider:

- Time-based partitioning for very large datasets
- Dedicated read-replicas for visualization-heavy workloads
- External time-series database for extreme scale

## Future Enhancements

Potential areas for expansion:

1. Authentication and user management
2. More sophisticated data visualization options
3. Anomaly detection and alerting
4. Data aggregation and statistical analysis
5. Mobile app integration
6. Time-zone support for global deployments
7. Expanded export formats
8. Report generation

## License

This project is licensed under the MIT License.