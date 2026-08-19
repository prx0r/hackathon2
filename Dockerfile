FROM rust:1.82-bookworm AS build
WORKDIR /app
COPY . .
RUN cargo build --release --workspace

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=build /app/target/release/iolaus-bench /usr/local/bin/iolaus-bench
COPY --from=build /app/target/release/iolaus-demo /usr/local/bin/iolaus-demo
WORKDIR /workspace
ENTRYPOINT ["iolaus-bench"]
