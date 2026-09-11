FROM rust:1.98.0-alpine AS builder
WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY src ./src

# This caching helps with the jenkins builds so they don't have to recompile all the deps all the time.
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/app/target \
    cargo build --release --locked && \
    cp target/release/military-portal /military-portal

FROM alpine:3.24
LABEL org.opencontainers.image.source=https://github.com/Sijma/military-portal
WORKDIR /app
COPY --from=builder /military-portal /app/military-portal
CMD ["/app/military-portal"]
