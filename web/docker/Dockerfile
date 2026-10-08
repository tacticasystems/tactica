# syntax=docker/dockerfile:1.28

ARG NODE_VERSION
ARG PNPM_VERSION

FROM node:${NODE_VERSION?} AS builder

ARG PNPM_VERSION
ARG TARGETARCH
RUN --mount=type=cache,target=/root/.npm,id=web-npm-${TARGETARCH},sharing=locked \
    SHELL=bash PNPM_HOME=/opt/pnpm npx --yes get-pnpm

WORKDIR /app

# Copy dependency configs first to leverage layer caching
COPY pnpm-lock.yaml package.json pnpm-workspace.yaml ./
RUN --mount=type=cache,target=/pnpm/store,id=web-pnpm-${PNPM_VERSION}-${TARGETARCH},sharing=locked \
    /opt/pnpm/bin/pnpm install --frozen-lockfile --store-dir /pnpm/store --package-import-method=copy

# Copy full application code and build static assets
COPY . .
RUN /opt/pnpm/bin/pnpm run build

# ==========================================
# STAGE 2: Secure Production Container (Rocky Linux 10)
# ==========================================
FROM rockylinux/rockylinux:10-ubi AS run

# Install Caddy repository configuration and the binary
RUN dnf install -y ca-certificates 'dnf-command(copr)' \
    && update-ca-trust \
    && dnf copr enable -y @caddy/caddy \
    && dnf install -y caddy \
    && dnf clean all

# Create standard directories for application code
RUN mkdir -p /var/www/html /etc/caddy

# Copy Caddy config
COPY docker/Caddyfile /etc/caddy/Caddyfile

# Copy compiled SPA static files from build stage
COPY --from=builder /app/dist /var/www/html

# Expose HTTP port
EXPOSE 8090

# Run Caddy in the foreground
CMD ["caddy", "run", "--config", "/etc/caddy/Caddyfile"]
