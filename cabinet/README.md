# Cabinet

面向 AI Agent 的离散语义记忆检索系统。

## 核心特性

- **HSH 编码**：20-bit 结构化整数替代 768-dim 浮点向量
- **纯 CPU 部署**：无需 GPU，O(log n) 检索复杂度
- **可解释检索**：检索路径完全可审计（类别→簇→词）
- **增量更新**：仅追加写入，无需重建索引
- **三层记忆架构**：Token / Archive / Working Memory

## 技术栈

- Rust Core (高性能 + 内存安全)
- PyO3 Python 绑定
- SQLite / RocksDB 后端

## 快速开始

```rust
use cabinet_core::{Memory, Config};

fn main() -> anyhow::Result<()> {
    let mut mem = Memory::open(Config::new("./agent_memory.db"))?;
    mem.insert("用户明天下午3点开会，准备PPT。")?;
    let results = mem.query("会议准备", 5)?;
    for hit in results {
        println!("doc={}, score={:.3}", hit.doc_id, hit.score);
    }
    Ok(())
}
```

```python
import cabinet

mem = cabinet.Memory("./agent_memory.db")
mem.insert("用户明天下午3点开会，准备PPT。")
results = mem.query("会议准备", top_k=5)
for r in results:
    print(f"[{r.score:.2f}] {r.text if r.match_level >= 3 else '...'}")
```

## 项目结构

```
cabinet/
├── crates/
│   ├── cabinet-hsh/      # 编码层（HSH 20-bit 编码）
│   ├── cabinet-index/    # 索引层（B-tree + LSM）
│   ├── cabinet-store/    # 存储层（SQLite/RocksDB）
│   ├── cabinet-router/   # 路由层（RelRouter）
│   └── cabinet-core/     # 核心编排层
├── pycabinet/            # PyO3 Python 绑定
└── bench/                # 基准测试
```

## 文档

- [架构设计](docs/architecture.md)
- [技术路线](../技术路线支持.md)

## License

MIT OR Apache-2.0
