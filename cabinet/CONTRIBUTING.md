# 贡献指南

感谢您对 Cabinet 项目的兴趣！

## 开发环境

- Rust 1.72+（`rustup update`）
- Python 3.8+（如需构建 PyO3 绑定）
- SQLite3（开发依赖）

## 快速开始

```bash
# 克隆仓库
git clone https://github.com/yourname/cabinet
cd cabinet

# 运行 Rust 测试
cargo test --workspace

# 运行 clippy
cargo clippy --workspace -- -D warnings

# 格式化代码
cargo fmt

# 构建 CLI
cargo build --release -p cabinet-cli

# 测试 CLI
./target/release/cabinet encode "明天下午3点开会"
./target/release/cabinet insert "准备PPT"
./target/release/cabinet query "会议准备"
```

## 目录结构说明

- `crates/cabinet-hsh/` — 编码层（零 IO，纯计算）
- `crates/cabinet-index/` — 索引层（B-tree + LSM）
- `crates/cabinet-store/` — 存储层（SQLite/RocksDB）
- `crates/cabinet-router/` — 路由层（RelRouter）
- `crates/cabinet-core/` — 核心编排层（Memory API）
- `crates/cabinet-cli/` — 命令行工具
- `crates/cabinet-tools/` — 离线工具链（构建聚类中心/种子表）
- `pycabinet/` — PyO3 Python 绑定
- `examples/` — 示例代码和数据
- `docs/` — 文档和论文素材

## 测试规范

- 编码层单元测试覆盖率 > 95%
- 新增功能必须附带测试
- 所有测试通过 `cargo test` 后提交 PR

## 提交规范

- 使用 [Conventional Commits](https://conventionalcommits.org/)
- 示例：`feat(hsh): 添加常用词晋升阈值配置`

## 论文素材

开发过程中的设计决策、性能瓶颈、参数调优过程请同步记录到 `docs/paper/` 目录，用于后续论文写作。

## 许可证

MIT OR Apache-2.0
