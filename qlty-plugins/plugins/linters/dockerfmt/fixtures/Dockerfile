FROM node:22-bookworm-slim AS development

RUN rm -f /var/lib/apt/lists/packages.clickhouse.com_* /var/lib/dpkg/arch /var/lib/apt/lists/partial/packages.clickhouse.com_* && \
    apt-get clean && \
    apt-get autoclean && \
    apt-get update && \
    apt-get upgrade -y && \
    apt-get install -y --no-install-recommends build-essential python3 openssl curl ca-certificates gnupg