FROM lukemathwalker/cargo-chef:latest AS chef
WORKDIR /app

FROM chef AS planner
COPY ./Cargo.toml ./Cargo.lock ./
COPY . .
RUN cargo chef prepare

FROM chef AS builder
COPY --from=planner /app/recipe.json .
RUN cargo chef cook --release
COPY . .
RUN cargo build --release --bin qlty
RUN mv ./target/release/qlty ./qlty

FROM debian:bookworm-slim AS runtime
WORKDIR /app
COPY --from=builder /app/qlty /usr/local/bin/
ENTRYPOINT ["/usr/local/bin/qlty"]