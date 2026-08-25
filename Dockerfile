FROM rust:1.98-slim AS builder
WORKDIR /src
COPY Cargo.toml ./
COPY src ./src
RUN cargo build --release

FROM debian:bookworm-slim
RUN useradd --system --uid 10001 --create-home app
WORKDIR /work
COPY --from=builder /src/target/release/sky-recovery /usr/local/bin/sky-recovery
USER 10001
ENTRYPOINT ["sky-recovery"]
