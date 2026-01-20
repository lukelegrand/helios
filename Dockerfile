# Stage 1: Build Rust Binary
FROM rust:1.75 as rust-builder
WORKDIR /app
COPY src/data-engine-rust/ .
RUN cargo build --release

# Stage 2: Build TypeScript
FROM node:18 as ts-builder
WORKDIR /app
COPY src/crawler-ts/ .
RUN npm install && npm run build # assumes tsc is setup

# Stage 3: Final Ray Image
FROM rayproject/ray:2.9.0-py310

# Install Node in the Ray image
USER root
RUN apt-get update && apt-get install -y curl
RUN curl -fsSL https://deb.nodesource.com/setup_18.x | bash -
RUN apt-get install -y nodejs
# Install Playwright Dependencies
RUN npx playwright install-deps
RUN npx playwright install chromium

# Copy artifacts
COPY --from=rust-builder /app/target/release/helios-engine /app/target/release/helios-engine
COPY --from=ts-builder /app /app/src/crawler-ts

# Copy Python Orchestrator
COPY src/orchestrator /app/src/orchestrator

# Install Python deps
RUN pip install boto3

USER ray
WORKDIR /app