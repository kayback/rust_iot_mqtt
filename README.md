# IoT Telemetry Ingest System


IoT telemetry ingestion system built with Rust. Receives sensor data via MQTT, stores in PostgreSQL, and exposes metrics via Prometheus and REST API.

---

## ✨ Features

### Core Functionality
- ✅ **MQTT Ingestion** - Receives telemetry via MQTT (QoS 1)
- ✅ **Data Validation** - Schema validation with configurable ranges
- ✅ **Batch Processing** - Efficient bulk inserts (2000 records/batch)
- ✅ **PostgreSQL Storage** - Persistent storage
- ✅ **REST API** - Query telemetry with filters (device_id, time range)
- ✅ **Prometheus Metrics** - Comprehensive observability
- ✅ **Grafana Dashboard** - Pre-configured visualization

### Production Features
- ✅ **High Throughput** - 1500-2000 msg/s sustained
- ✅ **Low Latency** - <100ms p95 latency
- ✅ **Automatic Reconnection** - MQTT auto-reconnect on failure
- ✅ **Retry/Backoff** - Transient error handling
- ✅ **Structured Logging** - JSON logs with tracing
- ✅ **Docker Support** - Full containerization
- ✅ **Systemd Service** - Production deployment ready

---

## 📦 Prerequisites

### Required

- **Rust** 1.70+
- **Docker** & **Docker Compose**
- **PostgreSQL** 14+ (or via Docker)
- **MQTT Broker** (Mosquitto recommended, or via Docker)
- **Prometheus** (for metrics, included in docker-compose)
- **Grafana** (for dashboards, included in docker-compose)


---

## 🚀 Quick Start

```bash
# Clone repository
git clone https://github.com/kayback/rust_iot_mqtt.git
cd rust_iot_mqtt
```

### Option 1. Use Docker Compose:

```bash
# Start all services (Ingestor, Simulator, PostgreSQL, Mosquitto, Prometheus, Grafana)
docker-compose -f deploy/docker-compose.yml up --build -d

# View logs
docker-compose logs -f ingestor

# Stop all services
docker-compose down

# Stop and remove volumes
docker-compose down -v
```

### Option 2. Use systemd:

```bash
# 1. Build release binary
cargo build --release

# 2. Install service (requires sudo)
sudo ./scripts/install-service.sh

# 3. Configure
sudo nano /etc/systemd/system/iot-ingestor.service

# 4. Run start-dependencies.sh:
sudo ./scripts/start-dependencies.sh

# 5. Enable service to start on boot:
sudo systemctl enable iot-ingestor

# 6. Start service
sudo systemctl start iot-ingestor

# 7. Check status
sudo systemctl status iot-ingestor

# 8. View logs
sudo journalctl -u iot-ingestor -f"
```

### Verify

```bash
# Query telemetry
curl http://localhost:8080/api/v1/telemetry?limit=10

# Check metrics
curl http://localhost:8080/metrics | grep ingestor_

# View Grafana dashboard
open http://localhost:3000  # admin/admin
```

---

## ⚙️ Configuration

### Environment Variables

#### Ingestor

| Variable | Default | Description |
|----------|---------|-------------|
| `DATABASE_URL` | `postgres://iot:pass@localhost:5432/iotdb` | PostgreSQL connection string |
| `MQTT_BROKER` | `localhost` | MQTT broker hostname |
| `MQTT_PORT` | `1883` | MQTT broker port |
| `HTTP_ADDR` | `0.0.0.0:8080` | HTTP server bind address |
| `BATCH_SIZE` | `2000` | Records per batch insert |
| `BATCH_TIMEOUT_MS` | `50` | Max wait before flush (ms) |
| `RUST_LOG` | `info` | Log level (trace/debug/info/warn/error) |

#### Simulator

| Variable | Default | Description |
|----------|---------|-------------|
| `MQTT_BROKER` | `localhost` | MQTT broker hostname |
| `MQTT_PORT` | `1883` | MQTT broker port |
| `NUM_DEVICES` | `10` | Number of simulated devices |
| `PUBLISH_INTERVAL_MS` | `1000` | Interval between messages (ms) |
| `RUST_LOG` | `info` | Log level |

### Example Configuration

```bash
export DATABASE_URL="postgres://iot:secure_pass@db.example.com:5432/iotdb"
export MQTT_BROKER="mqtt.example.com"
export MQTT_PORT="8883"
export HTTP_ADDR="0.0.0.0:8080"
export BATCH_SIZE="3000"
export BATCH_TIMEOUT_MS="30"
export RUST_LOG="info,ingestor=debug"
```

### Docker Compose Configuration

Edit `deploy/docker-compose.yml`:

```yaml
ingestor:
  environment:
    - BATCH_SIZE=3000
    - BATCH_TIMEOUT_MS=30
    - RUST_LOG=debug
```
---

## 📡 API Usage

### Base URL

```
http://localhost:8080
```

### Endpoints

---

#### 1. Query Telemetry

```bash
GET /api/v1/telemetry

# Query parameters:
#   device_id  - Filter by device ID
#   start      - Start time (ISO 8601)
#   end        - End time (ISO 8601)
#   limit      - Max records (default: 100, max: 1000)
#   offset     - Pagination offset (default: 0)
```

**Examples:**

```bash
# Get last 10 records
curl "http://localhost:8080/api/v1/telemetry?limit=10"

# Get records for specific device
curl "http://localhost:8080/api/v1/telemetry?device_id=sensor-001"

# Get records in time range
curl "http://localhost:8080/api/v1/telemetry?start=2025-10-05T00:00:00Z&end=2025-10-05T23:59:59Z"

# Combined filters
curl "http://localhost:8080/api/v1/telemetry?device_id=sensor-001&start=2025-10-05T10:00:00Z&limit=100"
```

**Response:**

```json
{
  "data": [
    {
      "device_id": "sensor-001",
      "timestamp": "2025-10-05T12:34:56Z",
      "temperature": 23.5,
      "humidity": 65.2,
      "battery": 87.3
    }
  ],
  "total": 1,
  "limit": 100,
  "offset": 0
}
```

---

#### 2. Prometheus Metrics

```bash
GET /metrics

curl http://localhost:8080/metrics
```

**Response:**
```
# HELP ingestor_messages_total Total MQTT messages received
# TYPE ingestor_messages_total counter
ingestor_messages_total 15234

# HELP ingestor_valid_messages_total Valid messages after validation
# TYPE ingestor_valid_messages_total counter
ingestor_valid_messages_total 15189

# ... more metrics
```

---

### API Error Responses

**400 Bad Request:**
```json
{
  "error": "Invalid time range"
}
```

**500 Internal Server Error:**
```json
{
  "error": "Database query failed: ..."
}
```

---

## 📊 Metrics & Monitoring

### Prometheus Metrics

| Metric | Type | Description |
|--------|------|-------------|
| `ingestor_messages_total` | Counter | Total MQTT messages received |
| `ingestor_valid_messages_total` | Counter | Valid messages after validation |
| `ingestor_invalid_messages_total` | Counter | Invalid messages rejected |
| `ingestor_db_inserts_total` | Counter | Successful database inserts |
| `ingestor_db_failures_total` | Counter | Failed database operations |
| `ingestor_batch_size` | Counter | Current batch size |
| `ingestor_ingest_latency_seconds` | Histogram | Batch insert latency |

### Grafana Dashboard

Access at `http://localhost:3000` (admin/admin)

**Pre-configured panels:**
- Message rate (msg/s)
- Validation success rate (%)
- Database insert rate
- Error rates
- Latency histogram (p50, p95, p99)
- System resources (CPU, memory)

**Import dashboard:**
1. Open Grafana → Dashboards → Import
2. Upload `deploy/grafana/dashboard.json`
3. Select Prometheus datasource
---

## 🏗️ Architecture

### System Overview

```
┌──────────────┐ 
│   Simulator  │   
│              │ 
└──────┬───────┘ 
       │                     
       │  MQTT (QoS 1)       
       │                      
       ▼                      
┌─────────────────────────────────────┐
│        MQTT Broker (Mosquitto)      │
└───────────────┬─────────────────────┘
                │
                │ Subscribe: telemetry/#
                ▼
┌─────────────────────────────────────┐
│         Ingestor Service            │
│  ┌──────────────────────────────┐   │
│  │  1. MQTT Handler (10K buffer)│   │
│  │  2. JSON Parser              │   │
│  │  3. Validator                │   │
│  │  4. Channel (50K capacity)   │   │
│  │  5. Batcher (2K records/50ms)│   │
│  │  6. DB Writer (Bulk INSERT)  │   │
│  └──────────────────────────────┘   │
└──┬──────────────────────────────┬───┘
   │                              │
   │                              │ Expose Metrics
   ▼                              ▼
┌─────────────┐         ┌──────────────┐
│ PostgreSQL  │         │  Prometheus  │
│  - Telemetry│         │   Grafana    │
│  - Indexes  │         └──────────────┘
└─────────────┘

REST API: http://localhost:8080/api/v1/telemetry
```

### Data Flow

```
MQTT Message → Parse JSON → Validate → Channel → Batch → PostgreSQL
                  ↓            ↓          ↓        ↓         ↓
              Invalid?    Out of range?  Full?   Flush?   Error?
                  ↓            ↓          ↓        ↓         ↓
              Log+Metric   Log+Metric  Retry  Insert   Retry+Log
```

### Database Schema

```sql
CREATE TABLE telemetry (
    id BIGSERIAL PRIMARY KEY,
    device_id TEXT NOT NULL,
    ts TIMESTAMPTZ NOT NULL,
    temperature DOUBLE PRECISION NOT NULL,
    humidity DOUBLE PRECISION NOT NULL,
    battery DOUBLE PRECISION NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE(device_id, ts)
);

CREATE INDEX idx_telemetry_device_time 
    ON telemetry(device_id, ts DESC);
    
CREATE INDEX idx_telemetry_created 
    ON telemetry(created_at DESC);
```
---

## 🔧 Troubleshooting

### Common Issues

#### 1. Cannot connect to MQTT broker

**Error:** `Connection refused (os error 111)`

**Solution:**
```bash
# Check if Mosquitto is running
docker-compose ps | grep mosquitto

# Check logs
docker-compose logs mosquitto

# Restart broker
docker-compose restart mosquitto

# Test connection
mosquitto_sub -h localhost -p 1883 -t test/#
```

---

#### 2. Database connection fails

**Error:** `Failed to connect to database: Connection refused`

**Solution:**
```bash
# Check if PostgreSQL is running
docker-compose ps | grep postgres

# Check logs
docker-compose logs postgres

# Test connection
docker exec iot-postgres psql -U iot -d iotdb -c "SELECT 1;"

# Reset database
docker-compose down -v
docker-compose up -d postgres
```
---

#### 3. High memory usage

**Symptom:** Ingestor using >500MB RAM

**Solution:**
```bash
# Reduce channel capacity
export CHANNEL_SIZE=20000  # Was 50000

# Reduce batch size
export BATCH_SIZE=1000  # Was 2000

# Restart ingestor
docker-compose restart ingestor
```

---

#### 4. Low throughput

**Symptom:** <500 msg/s throughput

**Check:**
```bash
# 1. Check MQTT broker
docker stats iot-mosquitto

# 2. Check database connections
docker exec iot-postgres psql -U iot -d iotdb \
  -c "SELECT count(*) FROM pg_stat_activity WHERE state='active';"

# 3. Check metrics
curl http://localhost:8080/metrics | grep rate

# 4. Check for backpressure
docker-compose logs ingestor | grep "Channel full"
```

**Solutions:**
- Increase `BATCH_SIZE` (faster DB writes)
- Decrease `BATCH_TIMEOUT_MS` (lower latency)
- Increase database connection pool
- Add more ingestor instances (horizontal scaling)

---

#### 5. API returns 500 error

**Error:** `Internal server error: Database query failed`

**Solution:**
```bash
# Check database status
docker exec iot-postgres psql -U iot -d iotdb -c "SELECT NOW();"

# Check for long-running queries
docker exec iot-postgres psql -U iot -d iotdb \
  -c "SELECT pid, now() - query_start as duration, query 
      FROM pg_stat_activity 
      WHERE state != 'idle' 
      ORDER BY duration DESC;"

# Check logs
docker-compose logs ingestor | grep -i error

# Restart if needed
docker-compose restart ingestor
```
---

## Grafana Screenshot
![Grafana](ss/image.png)
