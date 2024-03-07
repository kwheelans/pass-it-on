FROM lukemathwalker/cargo-chef:latest as chef

FROM chef AS planner
WORKDIR /recipe
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
WORKDIR /pass-it-on

# Build dependencies
COPY --from=planner /recipe/recipe.json recipe.json
RUN cargo chef cook --release --features server-bin-full,vendored-tls,bundled-sqlite --recipe-path recipe.json

# Build application
COPY ./ .
RUN cargo build --release --bin pass-it-on-server  --no-default-features --features server-bin-full,vendored-tls,bundled-sqlite

# Final image
FROM debian:12-slim

RUN mkdir /pass-it-on
WORKDIR /pass-it-on

ENV PATH=/pass-it-on:$PATH \
LOG_LEVEL=Info

ADD resources/docker/start_server.sh /pass-it-on/
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=builder /pass-it-on/target/release/pass-it-on-server /pass-it-on

VOLUME /config

CMD ["/bin/sh","start_server.sh"]
