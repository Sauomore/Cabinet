.DEFAULT_GOAL := help

.PHONY: help build test fmt clippy clean install python docker bench

help: ## 显示帮助信息
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | awk 'BEGIN {FS = ":.*?## "}; {printf "\033[36m%-20s\033[0m %s\n", $$1, $$2}'

build: ## 构建所有 Rust crate
	cargo build --workspace

test: ## 运行所有测试
	cargo test --workspace

fmt: ## 格式化代码
	cargo fmt --all

clippy: ## 运行 Clippy 检查
	cargo clippy --workspace -- -D warnings

clean: ## 清理构建产物
	cargo clean

install: ## 安装 CLI 到本地
	cargo install --path crates/cabinet-cli

python-dev: ## 构建并安装 Python 绑定（开发模式）
	cd pycabinet && maturin develop

python-build: ## 构建 Python wheel
	cd pycabinet && maturin build --release

python-test: ## 测试 Python 绑定
	python scripts/test-install.py

docker-build: ## 构建 Docker 镜像
	docker-compose build

docker-up: ## 启动 Docker 容器
	docker-compose up -d

bench: ## 运行基准测试
	cargo run --manifest-path bench/Cargo.toml --release

release: ## 完整发布流程（构建 + 测试 + 格式化检查）
	cargo fmt --all -- --check
	cargo clippy --workspace -- -D warnings
	cargo test --workspace
	cargo build --release --workspace
	@echo "Release build complete!"
