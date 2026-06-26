# PyPI发布指南

## 打包前准备

1. 更新版本号（修改 `Cargo.toml` 和 `pyproject.toml` 中的 version）
2. 确保 Rust 核心代码已编译通过：`cargo test --workspace`

## 本地构建测试

```bash
cd cabinet/pycabinet
pip install maturin
maturin develop
python -c "import pycabinet; print(pycabinet.__version__)"
```

## 发布到 PyPI（使用 cibuildwheel 推荐流程）

```bash
pip install cibuildwheel
# 构建当前平台 wheel
cibuildwheel --platform windows

# 上传（需要 PyPI token）
maturin publish --skip-existing
```

或使用 GitHub Actions（推荐）：
- 推送 tag 触发自动构建和发布

## 依赖说明

- **Rust 工具链**：仅开发/构建时需要。终端用户安装预编译 wheel 无需 Rust。
- **GUI 可选**：`pip install pycabinet[gui]` 额外安装 streamlit 等可视化依赖

## 安装方式

| 方式 | 命令 | 需要 Rust |
|------|------|-----------|
| 预编译 wheel | `pip install pycabinet` | ❌ 否 |
| 源码编译 | `pip install pycabinet --no-binary` | ✅ 是 |
| 含 GUI | `pip install pycabinet[gui]` | ❌ 否 |
| 开发安装 | `maturin develop` | ✅ 是 |

