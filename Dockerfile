FROM rust:1.99.1-slim-trixie AS builder

ARG TARGETARCH
WORKDIR /app
    
RUN apt-get update \
    && apt-get install -y --no-install-recommends curl ca-certificates pkg-config libssl-dev g++ \
    && rm -rf /var/lib/apt/lists/*

RUN case "$TARGETARCH" in \
        amd64) TW=tailwindcss-linux-x64 ;; \
        arm64) TW=tailwindcss-linux-arm64 ;; \
        *) echo "Unsupported architecture: $TARGETARCH" && exit 1 ;; \
    esac \
    && curl -fsSL -o /usr/local/bin/tailwind \
        "https://github.com/tailwindlabs/tailwindcss/releases/latest/download/$TW" \
    && chmod +x /usr/local/bin/tailwind

COPY Cargo.toml Cargo.lock* ./
RUN mkdir src && echo "fn main() {}" > src/main.rs && touch src/lib.rs \
        && cargo build --release \
        && rm -rf src
    
    
COPY askama.toml ./
COPY src ./src
COPY frontend ./frontend
    
RUN tailwind -m -i frontend/assets/input.css -o frontend/assets/styles.css

RUN touch src/main.rs src/lib.rs && cargo build --release

FROM debian:trixie-slim AS runtime
    
RUN apt-get update \
        && apt-get install -y --no-install-recommends ca-certificates libssl3t64 libstdc++6 \
        && rm -rf /var/lib/apt/lists/*
    
RUN useradd --system --uid 10001 --no-create-home appuser
    
WORKDIR /app
COPY --from=builder /app/target/release/copycat /app/copycat
COPY --from=builder /app/frontend ./frontend
    
USER appuser
ENV RUST_LOG=debug
EXPOSE 3000
    
CMD ["/app/copycat"]