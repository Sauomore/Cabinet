#!/usr/bin/env bash
set -e

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
cd "$SCRIPT_DIR/.."

echo "========================================="
echo "   Cabinet Release Builder"
echo "========================================="

# 检查工具
python3 --version || { echo "[错误] 未找到 Python"; exit 1; }
cargo --version || { echo "[错误] 未找到 Rust"; exit 1; }

# 安装构建工具
echo "[步骤 1/3] 安装构建工具..."
pip3 install maturin cibuildwheel

# 构建 sdist + wheel
echo "[步骤 2/3] 构建..."
cd pycabinet
maturin sdist
maturin build --release

# 测试安装
echo "[步骤 3/3] 测试安装..."
for whl in target/wheels/*.whl; do
    pip3 install --force-reinstall "$whl"
    python3 -c "import pycabinet; m = pycabinet.Memory('./test_release.db'); m.insert('test'); print('OK')"
    break
done

echo "[完成] 构建产物: $(pwd)/target/wheels/"
