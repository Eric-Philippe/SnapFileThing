# Multi-stage Dockerfile for SnapFileThing
# This Dockerfile builds a Rust backend and a Node.js frontend, optimizing for size and performance

# Stage 1: Frontend build
FROM node:18-alpine AS frontend-builder

# Set the working directory for the frontend build
WORKDIR /app/frontend

# Copy dependency files
COPY frontend/package*.json ./

# Install all dependencies (including dev for the build)
RUN npm ci

# Copy source code
COPY frontend/ ./

# Build the frontend
RUN npm run build

# Stage 2: Backend build
FROM rust:1.88-alpine AS builder

# Set the working directory for the backend build
WORKDIR /app

# Install build dependencies
RUN apk add --no-cache musl-dev

# Copy dependency files first for better caching
COPY backend/Cargo.toml backend/Cargo.lock ./

# Create a dummy main.rs to build dependencies
RUN mkdir src && echo "fn main() {}" > src/main.rs

# Build dependencies (this layer will be cached if Cargo.toml doesn't change)
RUN cargo build --release && rm -rf src target/release/deps/snapfilething*

# Copy source code
COPY ./backend/src ./src

# Build the application with optimizations
RUN cargo build --release --locked \
    && strip target/release/snapfilething

# Runtime stage - ultra lightweight
FROM alpine:3.19 AS runtime
WORKDIR /app/backend

# Install only essential runtime dependencies
RUN apk add --no-cache ca-certificates libgcc

# Create uploads directory with proper permissions
RUN mkdir -p /uploads && chmod 755 /uploads

# Copy the frontend build from the frontend-builder stage
COPY --from=frontend-builder /app/frontend/dist /app/backend/../frontend/dist

# Copy the binary from builder stage
COPY --from=builder /app/target/release/snapfilething /app/backend/snapfilething
RUN chmod +x /app/backend/snapfilething

# Set uploads dir to the directory mounted in docker-compose
ENV UPLOAD_DIR=/uploads

ENTRYPOINT ["/app/backend/snapfilething"]