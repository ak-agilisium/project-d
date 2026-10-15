# syntax=docker/dockerfile:1

# ---- Base ----------------------------------------------------------------
# node:22-slim (Debian bookworm-slim): much smaller than the full image,
# glibc-based so it avoids the musl surprises Alpine can cause with native deps.
FROM node:22-slim AS base
ENV PNPM_HOME="/pnpm" \
    PATH="/pnpm:$PATH"
# Enable the pnpm shim that ships with Node's corepack.
RUN corepack enable

# ---- Dependencies --------------------------------------------------------
# Isolated layer so deps are only re-installed when the lockfile changes.
FROM base AS deps
WORKDIR /app
COPY package.json pnpm-lock.yaml pnpm-workspace.yaml ./
RUN --mount=type=cache,id=pnpm,target=/pnpm/store \
    pnpm install --frozen-lockfile

# ---- Builder -------------------------------------------------------------
FROM base AS builder
WORKDIR /app
ENV NEXT_TELEMETRY_DISABLED=1
COPY --from=deps /app/node_modules ./node_modules
COPY . .
RUN pnpm run build

# ---- Runner --------------------------------------------------------------
# Final image ships ONLY the standalone server + static assets — no sources,
# no dev deps, no full node_modules.
FROM node:22-slim AS runner
WORKDIR /app
ENV NODE_ENV=production \
    NEXT_TELEMETRY_DISABLED=1 \
    PORT=3000 \
    HOSTNAME=0.0.0.0

# Run as an unprivileged user.
RUN groupadd --system --gid 1001 nodejs \
    && useradd --system --uid 1001 --gid nodejs nextjs

# Static files that the standalone server expects to serve.
COPY --from=builder /app/public ./public
# Standalone output includes a minimal node_modules + server.js.
COPY --from=builder --chown=nextjs:nodejs /app/.next/standalone ./
COPY --from=builder --chown=nextjs:nodejs /app/.next/static ./.next/static

USER nextjs
EXPOSE 3000

CMD ["node", "server.js"]
