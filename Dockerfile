FROM rust:1.99.1-slim AS builder

ARG TARGETARCH
WORKDIR /app


RUN apt-get update \
    && apt-get install -y --no-install-recommends curl ca-certificates \
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
RUN mkdir src && echo "fn main() {}" > src/main.rs \
      && cargo build --release \
      && rm -rf src


COPY askama.toml ./
COPY src ./src
COPY frontend ./frontend

RUN tailwind -m -i frontend/assets/input.css -o frontend/assets/styles.css

RUN touch src/main.rs && cargo build --release

FROM debian:bookworm-slim AS runtime

RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/*

RUN useradd --system --uid 10001 --no-create-home appuser

WORKDIR /app
COPY --from=builder /app/target/release/copycat /app/copycat
COPY --from=builder /app/frontend ./frontend

# COPY configuration ./configuration
# COPY data ./data # TODO delete, use volume instead

USER appuser

ENV RUST_LOG=info
EXPOSE 3000

CMD ["/app/copycat"]