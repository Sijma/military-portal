FROM rust:1.98.0-alpine AS builder
WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY src ./src
RUN cargo build --release --locked

FROM alpine:3.24
WORKDIR /app
COPY --from=builder /app/target/release/military-portal /app/military-portal
CMD ["/app/military-portal"]
