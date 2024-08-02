FROM rust:1.74-slim-bullseye AS chef
# We only pay the installation cost once, it will be cached from the second build onwards.
# It is the same version as credimi/diesel-migrations: so, keep both versions aligned for caching.
RUN cargo install cargo-chef
WORKDIR /app

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef as builder
RUN apt-get -y update \
    && apt-get install -y --no-install-recommends ca-certificates build-essential pkg-config libssl-dev libpq-dev \
    && apt-get clean \
    && rm -rf /var/lib/apt/lists/*
COPY --from=planner /app/recipe.json recipe.json
# Build dependencies - this is the caching Docker layer!
RUN cargo chef cook --release --recipe-path recipe.json
# Build application
COPY . .
RUN cargo build --release
RUN cargo test --release --no-run

FROM credimi/diesel-migrations:60cc08a as migrations
CMD ["./run_migrations.sh"]

FROM debian:bullseye-slim as runtime
RUN apt-get -y update \
    && apt-get install -y --no-install-recommends ca-certificates libpq5 \
    && apt-get clean \
    && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/http_server /usr/local/bin/app
ENTRYPOINT ["/usr/local/bin/app"]
