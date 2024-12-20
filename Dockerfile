FROM lukemathwalker/cargo-chef:latest AS chef
WORKDIR /app

FROM chef AS planner
COPY . .
# More on how cargo-chef works:
# https://github.com/LukeMathWalker/cargo-chef?tab=readme-ov-file#benefits-vs-limitations
RUN cargo chef prepare

FROM chef AS builder
COPY --from=planner /app/recipe.json .
# The line compiles dependencies. Since it's before the COPY line
# it will be cached and not re-run unless the dependencies change.
RUN cargo chef cook --release
COPY . .
# The line compiles the application, and will re-run every code change
RUN cargo build --release --bin qlty
RUN mv ./target/release/qlty ./qlty

FROM debian:bookworm-slim AS runtime
WORKDIR /app
COPY --from=builder /app/qlty /usr/local/bin/
ENTRYPOINT ["/usr/local/bin/qlty"]

# Example usage:
#
# docker run --rm -it -v "$(pwd):/app" qltysh/qlty metrics --all