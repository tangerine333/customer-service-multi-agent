FROM python:3.11-slim AS builder

WORKDIR /app
RUN apt-get update && apt-get install -y --no-install-recommends \
    build-essential curl git && \
    rm -rf /var/lib/apt/lists/*

# Install Rust toolchain for building rust_analyzer
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
ENV PATH="/root/.cargo/bin:${PATH}"

# Python dependencies
COPY requirements.txt .
RUN pip install --no-cache-dir -r requirements.txt

# Build Rust extension
COPY rust_analyzer/ rust_analyzer/
RUN cd rust_analyzer && \
    cargo build --release && \
    cp target/release/librust_analyzer.so /app/rust_analyzer.so

# --- Runtime ---
FROM python:3.11-slim

WORKDIR /app

RUN apt-get update && apt-get install -y --no-install-recommends \
    git libgomp1 ca-certificates && \
    rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/rust_analyzer.so /app/
COPY --from=builder /usr/local/lib/python3.11/site-packages /usr/local/lib/python3.11/site-packages
COPY requirements.txt .
RUN pip install --no-cache-dir -r requirements.txt

COPY src/ src/
COPY prompts/ prompts/
COPY tests/ tests/

ENV PYTHONUNBUFFERED=1
ENV CI_MODE=false

EXPOSE 8080

CMD ["uvicorn", "src.api.main:app", "--host", "0.0.0.0", "--port", "8080"]
