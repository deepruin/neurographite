# 🐳 NeuroGraphite Docker Deployment

Deploy NeuroGraphite as a containerized application with Docker and Docker Compose.

## 🚀 Quick Start

### Using Docker Compose (Recommended)

```bash
# Clone the repository
git clone https://github.com/deepruin/neurographite.git
cd neurographite

# Start the service
docker-compose up -d

# Check logs
docker-compose logs -f

# Stop the service
docker-compose down
```

### Using Docker Run

```bash
# Build the image
./build-docker.sh

# Run the container
docker run -d \
  --name neurographite \
  -p 8080:8080 \
  -v neurographite-data:/app/data \
  deepruin/neurographite:latest

# Check status
docker ps
docker logs neurographite
```

### From DockerHub (Coming Soon)

```bash
# Pull and run from DockerHub
docker run -d \
  --name neurographite \
  -p 8080:8080 \
  -v neurographite-data:/app/data \
  deepruin/neurographite:latest
```

## 🔧 Configuration

### Environment Variables

| Variable | Default | Description |
|----------|---------|-------------|
| `NEUROGRAPHITE_HOST` | `0.0.0.0` | Bind address |
| `NEUROGRAPHITE_PORT` | `8080` | HTTP port |
| `NEUROGRAPHITE_DATA_DIR` | `/app/data` | Data directory |
| `RUST_LOG` | `info` | Log level (trace, debug, info, warn, error) |

### Volume Mounts

- **Data persistence:** `-v neurographite-data:/app/data`
- **Config files:** `-v ./config:/app/config:ro` (optional)
- **Custom frontend:** `-v ./custom-frontend:/app/frontend:ro` (optional)

## 🏗️ Building Images

### Development Build

```bash
# Build with custom tag
./build-docker.sh v0.1.0

# Build latest
./build-docker.sh
```

### Production Build

```bash
# Build optimized image
docker build -t deepruin/neurographite:latest .

# Multi-platform build (requires buildx)
docker buildx build --platform linux/amd64,linux/arm64 \
  -t deepruin/neurographite:latest .
```

## 🚀 Deployment Options

### Docker Compose Production

```yaml
version: '3.8'
services:
  neurographite:
    image: deepruin/neurographite:latest
    ports:
      - "80:8080"
    volumes:
      - /opt/neurographite/data:/app/data
      - /opt/neurographite/config:/app/config:ro
    environment:
      - RUST_LOG=warn
      - NEUROGRAPHITE_HOST=0.0.0.0
    restart: unless-stopped
    deploy:
      resources:
        limits:
          memory: 1G
          cpus: '0.5'
```

### Kubernetes Deployment

```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: neurographite
spec:
  replicas: 1
  selector:
    matchLabels:
      app: neurographite
  template:
    metadata:
      labels:
        app: neurographite
    spec:
      containers:
      - name: neurographite
        image: deepruin/neurographite:latest
        ports:
        - containerPort: 8080
        env:
        - name: RUST_LOG
          value: "info"
        volumeMounts:
        - name: data
          mountPath: /app/data
        resources:
          limits:
            memory: "1Gi"
            cpu: "500m"
          requests:
            memory: "512Mi"
            cpu: "250m"
      volumes:
      - name: data
        persistentVolumeClaim:
          claimName: neurographite-pvc
```

### Docker Swarm

```yaml
version: '3.8'
services:
  neurographite:
    image: deepruin/neurographite:latest
    ports:
      - "8080:8080"
    volumes:
      - neurographite-data:/app/data
    environment:
      - RUST_LOG=info
    deploy:
      replicas: 2
      restart_policy:
        condition: on-failure
      resources:
        limits:
          memory: 1G
        reservations:
          memory: 512M
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:8080/health"]
      interval: 30s
      timeout: 10s
      retries: 3

volumes:
  neurographite-data:
    driver: local
```

## 🔍 Health Checks

The container includes built-in health checks:

```bash
# Check container health
docker inspect --format='{{.State.Health.Status}}' neurographite

# Manual health check
curl http://localhost:8080/health

# Expected response
{
  "status": "healthy",
  "timestamp": "2026-02-16T00:00:00Z",
  "version": "0.1.0"
}
```

## 📊 Monitoring

### Docker Stats

```bash
# Container resource usage
docker stats neurographite

# Detailed inspection
docker inspect neurographite
```

### Logs

```bash
# Follow logs
docker logs -f neurographite

# Last 100 lines
docker logs --tail 100 neurographite

# Logs with timestamps
docker logs -t neurographite
```

## 🛠️ Troubleshooting

### Common Issues

**Port already in use:**
```bash
# Find process using port 8080
lsof -i :8080
kill -9 <PID>
```

**Permission issues:**
```bash
# Fix data directory permissions
sudo chown -R 1000:1000 ./data
```

**Memory issues:**
```bash
# Check container resources
docker stats neurographite

# Increase memory limit
docker run --memory=2g deepruin/neurographite:latest
```

### Debug Mode

```bash
# Run with debug logging
docker run -e RUST_LOG=debug deepruin/neurographite:latest

# Interactive debug session
docker run -it --entrypoint /bin/bash deepruin/neurographite:latest
```

## 🔒 Security

### Production Security

- **Non-root user:** Container runs as `neurographite` user (UID 1000)
- **Read-only filesystem:** Mount `/app/data` as the only writable volume
- **Network isolation:** Use Docker networks for service communication
- **Resource limits:** Set memory and CPU limits
- **Health checks:** Automatic container restart on failures

### Secure Configuration

```bash
# Run with minimal privileges
docker run --read-only --tmpfs /tmp \
  -v neurographite-data:/app/data \
  deepruin/neurographite:latest

# Network isolation
docker network create neurographite-net
docker run --network neurographite-net deepruin/neurographite:latest
```

---

**🧠 Ready to deploy your neuromorphic hypergraph database anywhere!**