# Changelog

所有项目的显著变更都将记录在此文件中。

格式基于 [Keep a Changelog](https://keepachangelog.com/zh-CN/1.0.0/)，
版本号遵循 [Semantic Versioning](https://semver.org/lang/zh-CN/)。

## [Unreleased]

### Added
- 项目骨架搭建（Cargo workspace + 6 个 crate）
- HSH 20-bit 编码层（cabinet-hsh）：分词、POS 映射、聚类中心、准完美哈希
- 索引层（cabinet-index）：B-tree 前缀索引、VByte/Delta 压缩、PostingList
- 存储层（cabinet-store）：SQLite 后端、WAL 崩溃恢复
- 路由层（cabinet-router）：硬编码关联矩阵 + RelRouter 扩展点
- 核心编排层（cabinet-core）：Memory API、三层记忆架构、四级检索
- PyO3 Python 绑定（pycabinet）：`Memory.insert` / `Memory.query` / `Memory.decode`
- CLI 工具（cabinet-cli）：`insert` / `query` / `batch` / `stats` / `snapshot` / `encode`
- 离线工具链（cabinet-tools）：聚类中心构建、种子表搜索
- Streamlit 可视化 GUI（cabinet-gui）：编码可视化、三层架构图、检索路径、索引浏览器
- Docker 支持（Dockerfile + docker-compose.yml）
- 配置文件支持（config.toml）
- 示例代码（Rust + Python）和示例语料
- GitHub Actions CI/CD：自动测试 + 自动发布 PyPI wheel

## [0.1.0] — 2026-06-25

- MVP 初始版本，支持 Light 精度模式
- 纯 CPU 部署，无需 GPU
- 支持中文文本分词和 HSH 编码
- 支持 SQLite 后端存储
- 支持 Python 3.8–3.12

[Unreleased]: https://github.com/yourname/cabinet/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/yourname/cabinet/releases/tag/v0.1.0
