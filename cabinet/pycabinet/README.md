# pycabinet

> Python bindings for **Cabinet** — Hierarchical Semantic Hashing (HSH) memory retrieval system for AI agents.

## 快速开始

```bash
pip install pycabinet
```

```python
import pycabinet

# 初始化记忆库
mem = pycabinet.Memory(
    path="./agent_memory.db",
    precision="light",
    pos_threshold=50,
    max_context=4096
)

# 插入记忆
mem.insert("用户明天下午3点开会，准备PPT。")
mem.insert("用户喜欢听管弦乐。")

# 检索记忆
results = mem.query("会议准备", top_k=5)
for r in results:
    print(f"[{r.score:.2f}] doc_id={r.doc_id} match_level={r.match_level}")
    if r.match_level >= 3:
        text = mem.decode(r)
        print(f"  text: {text}")

# 快照备份
mem.snapshot("./backup/2026-06-25.db")
mem.close()
```

## 安装方式

| 方式 | 命令 | 说明 |
|------|------|------|
| 标准安装 | `pip install pycabinet` | 下载预编译 wheel，无需 Rust |
| 含 GUI | `pip install pycabinet[gui]` | 额外安装可视化界面依赖 |
| 开发编译 | `maturin develop` | 从源码编译，需要 Rust 工具链 |

## 核心特性

- **20-bit HSH 编码**：用结构化整数替代 768-dim 浮点向量
- **纯 CPU 部署**：无需 GPU，O(log n) 检索复杂度
- **增量更新**：仅追加写入，无需重建索引
- **可解释检索**：检索路径完全可审计（类别→簇→词）
- **三层记忆架构**：Token / Archive / Working Memory

## 可选 GUI 可视化

```bash
pip install pycabinet[gui]
```

安装后运行可视化界面：
```bash
cabinet-gui
```

## 系统要求

- Python ≥ 3.8（CPython 3.8 / 3.9 / 3.10 / 3.11 / 3.12）
- Windows / macOS / Linux（x86_64, aarch64）

> **预编译 wheel**：支持上述平台，无需额外安装 Rust。  
> **源码编译**：需要 [Rust 工具链](https://rustup.rs/)（1.72+）。

## 架构

```
pycabinet (Python API)
  └── PyO3 绑定
      └── cabinet-core (Rust 核心)
          ├── cabinet-hsh     (20-bit HSH 编码)
          ├── cabinet-index   (B-tree 索引 + LSM)
          ├── cabinet-store   (SQLite 后端)
          └── cabinet-router  (关联路由)
```

## 许可

MIT OR Apache-2.0
