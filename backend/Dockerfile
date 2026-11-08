# syntax=docker/dockerfile:1
# Build from the repo root: docker build -f backend/Dockerfile .
# One image: Rust API + the built Vue frontend as static files.

FROM node:20-alpine AS frontend
WORKDIR /app
COPY frontend/package.json frontend/package-lock.json ./
RUN npm ci
COPY frontend/ .
ARG VITE_RAILS_API_URL
ENV VITE_RAILS_API_URL=${VITE_RAILS_API_URL}
RUN npm run build

FROM rust:1.99-slim-bookworm AS backend
WORKDIR /app
COPY backend/Cargo.toml backend/Cargo.lock ./
COPY backend/src ./src
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/app/target \
    cargo build --release --locked && cp target/release/gravity-tasks-backend /app/gravity-tasks

FROM gcr.io/distroless/cc-debian12
WORKDIR /app
COPY --from=backend /app/gravity-tasks /app/gravity-tasks
COPY --from=frontend /app/dist /app/public
ENV DATABASE_PATH=/app/storage/production.sqlite3 \
    STATIC_DIR=/app/public \
    PORT=3000
# Same uid as the old Rails image, so the existing storage files stay writable.
USER 1000:1000
EXPOSE 3000
CMD ["/app/gravity-tasks"]
