#!/bin/bash
set -e

# NeuroGraphite Docker Build Script
# Usage: ./build-docker.sh [tag]

# Default tag
TAG=${1:-latest}
IMAGE_NAME="deepruin/neurographite"
FULL_TAG="${IMAGE_NAME}:${TAG}"

echo "🧠 Building NeuroGraphite Docker Image"
echo "Image: ${FULL_TAG}"
echo "----------------------------------------"

# Build the image
echo "📦 Building Docker image..."
docker build -t ${FULL_TAG} .

# Also tag as latest if not already
if [ "${TAG}" != "latest" ]; then
    docker tag ${FULL_TAG} ${IMAGE_NAME}:latest
    echo "✅ Tagged as ${IMAGE_NAME}:latest"
fi

echo "✅ Build complete: ${FULL_TAG}"
echo ""
echo "🚀 Next steps:"
echo "  Local run:    docker-compose up"
echo "  Push to hub:  docker push ${FULL_TAG}"
echo "  Test health:  curl http://localhost:8080/health"
echo ""
echo "🔧 Environment variables:"
echo "  NEUROGRAPHITE_HOST=0.0.0.0"
echo "  NEUROGRAPHITE_PORT=8080"
echo "  NEUROGRAPHITE_DATA_DIR=/app/data"
echo "  RUST_LOG=info"