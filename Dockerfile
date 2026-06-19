FROM rust:1.74-slim AS builder
WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim
WORKDIR /app
COPY --from=builder /app/target/release/rust-disaster-recovery .
EXPOSE 8080
CMD ["./rust-disaster-recovery"]
