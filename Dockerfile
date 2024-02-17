FROM lukemathwalker/cargo-chef:latest as chef

FROM chef AS planner
WORKDIR /recipe
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
WORKDIR /pass-it-on

# Build dependencies
COPY --from=planner /recipe/recipe.json recipe.json
RUN cargo chef cook --release --features server-bin-full,vendored-tls --recipe-path recipe.json

# Build application
COPY ./ .
RUN cargo build --release --bin pass-it-on-server  --no-default-features --features server-bin-full,vendored-tls

# Final image
FROM debian:12-slim

RUN mkdir /pass-it-on /config
WORKDIR /pass-it-on

ENV PATH=/pass-it-on:$PATH \
LOG_LEVEL=Info

COPY --from=builder /pass-it-on/target/release/pass-it-on-server /pass-it-on
ADD resources/docker/start_server.sh /pass-it-on/
ADD resources/docker/default_server_config.toml /config/server.toml
VOLUME /config

CMD ["/bin/sh","start_server.sh"]
