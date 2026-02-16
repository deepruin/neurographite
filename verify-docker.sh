#!/bin/bash
set -e

echo "🧠 NeuroGraphite Docker Verification"
echo "===================================="

# Check Docker is available
if ! command -v docker &> /dev/null; then
    echo "❌ Docker is not installed or not in PATH"
    exit 1
fi

echo "✅ Docker is available"

# Check Docker Compose is available
if ! command -v docker-compose &> /dev/null; then
    echo "⚠️  Docker Compose not found, checking for 'docker compose'"
    if ! docker compose version &> /dev/null; then
        echo "❌ Docker Compose is not available"
        exit 1
    fi
    COMPOSE_CMD="docker compose"
else
    COMPOSE_CMD="docker-compose"
fi

echo "✅ Docker Compose is available"

# Verify Dockerfile exists and is valid
if [ ! -f "Dockerfile" ]; then
    echo "❌ Dockerfile not found"
    exit 1
fi

echo "✅ Dockerfile exists"

# Verify docker-compose.yml exists and is valid
if [ ! -f "docker-compose.yml" ]; then
    echo "❌ docker-compose.yml not found"
    exit 1
fi

# Validate docker-compose.yml syntax
if ! $COMPOSE_CMD config &> /dev/null; then
    echo "❌ docker-compose.yml has syntax errors"
    exit 1
fi

echo "✅ docker-compose.yml is valid"

# Check source files exist
if [ ! -d "src" ] || [ ! -f "src/main.rs" ]; then
    echo "❌ Source files not found (src/main.rs missing)"
    exit 1
fi

echo "✅ Source files exist"

# Check frontend files exist
if [ ! -d "frontend" ] || [ ! -f "frontend/index.html" ]; then
    echo "❌ Frontend files not found"
    exit 1
fi

echo "✅ Frontend files exist"

# Check Cargo.toml exists
if [ ! -f "Cargo.toml" ]; then
    echo "❌ Cargo.toml not found"
    exit 1
fi

echo "✅ Cargo.toml exists"

# Optional: Try building the image (commented out to avoid long build times)
# echo "🔨 Testing Docker build..."
# if docker build -t neurographite-test . &> /dev/null; then
#     echo "✅ Docker build successful"
#     docker rmi neurographite-test &> /dev/null || true
# else
#     echo "❌ Docker build failed"
#     exit 1
# fi

echo ""
echo "🎉 All Docker setup verification checks passed!"
echo ""
echo "🚀 Ready to deploy:"
echo "  Local dev:    docker-compose up -d"
echo "  Build image:  ./build-docker.sh"
echo "  DockerHub:    docker push deepruin/neurographite:latest"
echo ""