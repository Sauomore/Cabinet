FROM rust:1.78-slim AS builder

WORKDIR /app

# 安装依赖
RUN apt-get update && apt-get install -y \
    python3 python3-pip python3-dev \
    libssl-dev pkg-config \
    && rm -rf /var/lib/apt/lists/*

# 复制项目
COPY Cargo.toml Cargo.lock ./
COPY crates/ ./crates/
COPY pycabinet/ ./pycabinet/

# 构建 CLI 和 core
RUN cargo build --release -p cabinet-cli -p cabinet-tools

# 构建 Python wheel (可选)
RUN pip3 install maturin && \
    cd pycabinet && \
    maturin build --release

# --- 运行时镜像 ---
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y \
    python3 python3-pip \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# 复制二进制
COPY --from=builder /app/target/release/cabinet /usr/local/bin/
COPY --from=builder /app/target/release/cabinet-build /usr/local/bin/

# 复制 Python wheel (可选安装)
COPY --from=builder /app/pycabinet/target/wheels/*.whl /tmp/

# 安装 pycabinet (可选)
RUN pip3 install /tmp/*.whl || true

# 数据卷
VOLUME ["/data"]
ENV CABINET_PATH=/data/agent_memory.db

ENTRYPOINT ["cabinet"]
CMD ["--help"]
