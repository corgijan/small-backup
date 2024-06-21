FROM lukemathwalker/cargo-chef:latest as chef
WORKDIR /app
ARG app_name=smbackup

FROM chef AS planner
COPY ./Cargo.toml ./Cargo.lock ./
COPY ./src ./src
RUN cargo chef prepare

FROM chef AS builder
COPY --from=planner /app/recipe.json .
RUN cargo chef cook --release
COPY . .
RUN cargo build --release
RUN ls -la ./target/release
RUN ls -la .
RUN mv ./target/release/smbackup /app/app

FROM debian:stable-slim AS runtime
WORKDIR /app
RUN mkdir /data
ENV REPLICATION_LOCATIONS=/data
ENV GENERATE_DIRS=True
COPY --from=builder /app/app /usr/local/bin/
ENTRYPOINT ["/usr/local/bin/app"]