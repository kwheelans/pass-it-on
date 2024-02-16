FROM lukemathwalker/cargo-chef:latest as chef

FROM chef AS planner
WORKDIR /recipe
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
WORKDIR /pass-it-on
RUN apk --no-cache add build-base

# Build dependencies
COPY --from=planner /recipe/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json

# Build application
COPY ./ .
RUN cargo build --release

# Final image
FROM debian:12-slim

RUN mkdir /pass-it-on
WORKDIR /pass-it-on

ENV PATH=/pass-it-on:$PATH

COPY --from=builder /pass-it-on/target/release/pass-it-on /pass-it-on

#CMD ["/bin/sh","setup.sh"]
