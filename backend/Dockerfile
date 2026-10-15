# syntax=docker/dockerfile:1

# =============================================================================
# Build stage
#   Pinned to Debian *bookworm* so the compiled binary's glibc matches the
#   distroless runtime (also bookworm/glibc 2.36).
# =============================================================================
FROM rust:1-slim-bookworm AS builder
WORKDIR /app

# Build toolchain:
#   - cmake + build-essential: aws-lc-sys (AWS SDK's rustls backend) and ring
#     compile C/asm at build time.
#   - ca-certificates: fetch crates over HTTPS.
# (All of this lives only in the builder and is discarded from the final image.)
RUN apt-get update \
    && apt-get install -y --no-install-recommends build-essential cmake ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Corporate networks (e.g. Zscaler) intercept outbound TLS and re-sign it with
# their own CA, which makes cargo's crates.io fetch fail SSL verification inside
# the container. Any *.crt dropped into ./certs is added to the trust store so
# the build works on such networks. The dir only holds a .gitkeep on CI/prod, so
# this is a harmless no-op there.
COPY certs/ /usr/local/share/ca-certificates/
RUN update-ca-certificates

# 1) Pre-compile dependencies against stub sources. This layer is cached and
#    only re-runs when Cargo.toml / Cargo.lock change — not on every code edit.
COPY Cargo.toml Cargo.lock ./
RUN mkdir src \
    && echo 'fn main() {}' > src/main.rs \
    && echo '//! stub' > src/lib.rs \
    && cargo build --release --locked \
    && rm -rf src

# 2) Real sources. openapi.yaml (include_str!) and migrations/ (sqlx::migrate!)
#    are embedded into the binary at COMPILE time, so both must be present now.
COPY src ./src
COPY migrations ./migrations
COPY openapi.yaml ./openapi.yaml
RUN touch src/main.rs src/lib.rs \
    && cargo build --release --locked

# =============================================================================
# Runtime stage
#   distroless/cc: ~20 MB, provides glibc + libgcc + ca-certificates, runs as a
#   non-root user (uid 65532), and ships no shell or package manager.
#   The release profile already strips the binary (see Cargo.toml).
# =============================================================================
FROM gcr.io/distroless/cc-debian12:nonroot AS runtime
WORKDIR /app

# Only the binary is needed at runtime — migrations and the OpenAPI spec are
# compiled into it; configuration is injected via the environment.
COPY --from=builder /app/target/release/opd-billing-backend /app/opd-billing-backend

EXPOSE 8080
# Config comes from the ECS task definition (env + secrets from Secrets Manager:
# project-d/backend/env). Do NOT bake a .env into the image.
ENTRYPOINT ["/app/opd-billing-backend"]
