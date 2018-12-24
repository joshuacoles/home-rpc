FROM shepmaster/rust-nightly:latest AS build

WORKDIR /compile
COPY . .

RUN cargo install --path .

ENV ROCKET_PORT 8000
ENV ROCKET_ADDRESS 0.0.0.0

ENTRYPOINT "home-rpc"
