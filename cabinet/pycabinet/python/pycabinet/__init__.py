# -*- coding: utf-8 -*-
"""pycabinet - Python bindings for Cabinet HSH memory retrieval."""

try:
    from pycabinet._pycabinet import (
        Memory, QueryResult, HSHCode, Encoder, MemoryStats, DrawerStats
    )
except ImportError as _e:
    raise ImportError(
        "pycabinet 的 Rust 扩展未编译。请先在项目根目录运行：\n"
        "  cd I:\\Cabinet_HSH\\库文件项目\\cabinet\n"
        "  maturin develop   # 开发安装（推荐）\n"
        "  # 或\n"
        "  maturin build --release   # 构建 wheel\n"
        f"原始错误: {_e}"
    ) from _e

__all__ = [
    "Memory",
    "QueryResult",
    "HSHCode",
    "Encoder",
    "MemoryStats",
    "DrawerStats",
    "plot",  # 延迟导入，见下
]


# plot 模块延迟导入，避免 matplotlib 硬依赖
class _LazyPlot:
    """延迟加载 plot 模块，仅在首次访问时导入 matplotlib。"""

    def __getattr__(self, name: str):
        from pycabinet import plot as _plot_module
        return getattr(_plot_module, name)


plot = _LazyPlot()
