#!/usr/bin/env bash
set -e

CONTAINER_NAME="rust_llvm15" # Docker container name
IMAGE_NAME="rust_llvm15_image"

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

if ! docker image inspect "$IMAGE_NAME" >/dev/null 2>&1; then
    echo "Building Docker image (rust_llvm15) ..."
    docker build -t "$IMAGE_NAME" "$ROOT_DIR"
fi

if ! docker ps -a --format '{{.Names}}' | grep -qw "$CONTAINER_NAME"; then
    echo "Creating container: $CONTAINER_NAME ..."
    docker run -dit \
        --name "$CONTAINER_NAME" \
        --network host \
        -v "$ROOT_DIR":/workspace \
        -w /workspace \
        "$IMAGE_NAME"
fi

# Run container (If not already running)
if [ "$(docker inspect -f '{{.State.Running}}' "$CONTAINER_NAME")" != "true" ]; then
    echo "Starting container: $CONTAINER_NAME ..."
    docker start "$CONTAINER_NAME" >/dev/null
fi

echo Attaching container ...
docker exec -it "$CONTAINER_NAME" /bin/bash