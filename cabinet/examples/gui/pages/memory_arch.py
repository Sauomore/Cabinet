# -*- coding: utf-8 -*-
"""记忆架构可视化 — 展示真实的 Token Store / Archive Index / Working Memory 状态。"""

import streamlit as st
import matplotlib.pyplot as plt
from matplotlib.patches import FancyBboxPatch
import pandas as pd

from pycabinet import Memory


@st.cache_resource
def get_memory():
    return Memory(
        path="./gui_memory.db",
        precision="light",
        pos_threshold=50,
        max_context=4096,
    )


FEAT_NAMES = {
    0x0: '名词', 0x1: '动词', 0x2: '形容词', 0x3: '副词',
    0x4: '代词', 0x5: '介词', 0x6: '连词', 0x7: '助词',
    0x8: '数词', 0x9: '量词', 0xA: '时间词', 0xB: '方位词',
    0xC: '标点', 0xD: '字符串', 0xE: '常用词', 0xF: '兜底',
}


def show():
    st.title("🗂️ 三层记忆架构可视化")
    st.markdown("Cabinet 模拟人类认知科学中的记忆分层，将记忆分为 Token、Archive、Working 三层。")

    mem = get_memory()

    try:
        stats = mem.get_stats()
    except Exception as e:
        st.error(f"获取统计失败: {e}")
        stats = None

    layer = st.selectbox(
        "选择层级深入查看",
        ["全部三层", "Layer 1: Token Store", "Layer 2: Archive Index", "Layer 3: Working Memory"]
    )

    if layer in ["全部三层", "Layer 1: Token Store"]:
        st.subheader("📄 Layer 1 — Token Store（词元层）")
        st.markdown("""
        - **作用**：存储原始文档的 HSH 序列，append-only
        - **结构**：内存追加缓冲区 + WAL（预写日志）
        - **触发**：缓冲区满 1000 条 → 后台 merge 到 Archive
        """)
        if stats:
            st.metric("当前缓冲区文档数", stats.token_store_buffer_size)
            st.metric("下一 doc_id", stats.next_doc_id)
        else:
            st.info("暂无统计信息")

    if layer in ["全部三层", "Layer 2: Archive Index"]:
        st.subheader("🗄️ Layer 2 — Archive Index（档案柜索引）")
        st.markdown("""
        - **作用**：按 16 个语义类别（feat）分组的倒排索引
        - **结构**：每个 feat 一个 Drawer → 内部 B-tree 按 (sim, abs) 索引
        - **压缩**：VByte + Delta 编码 PostingList
        """)

        # 绘制 16 抽屉图
        fig, ax = plt.subplots(figsize=(12, 6))
        drawer_names = [FEAT_NAMES.get(i, f'0x{i:01X}') for i in range(16)]
        colors = plt.cm.tab20(np.linspace(0, 1, 16))

        for i in range(16):
            row, col = i // 4, i % 4
            x, y = col * 3, 3 - row * 1.5
            rect = FancyBboxPatch((x, y), 2.5, 1.2, boxstyle="round,pad=0.1",
                                   facecolor=colors[i], edgecolor='black', alpha=0.8)
            ax.add_patch(rect)
            ax.text(x + 1.25, y + 0.75, f'0x{i:01X}', ha='center', va='center',
                    fontsize=10, fontweight='bold', color='white')
            ax.text(x + 1.25, y + 0.35, drawer_names[i], ha='center', va='center',
                    fontsize=8, color='white')

        ax.set_xlim(-0.5, 12)
        ax.set_ylim(-0.5, 4)
        ax.set_aspect('equal')
        ax.axis('off')
        ax.set_title('16 个 Feature Drawer（按 feat 分组）', fontsize=14, pad=20)
        st.pyplot(fig)

        # 真实 Drawer 统计
        st.markdown("**每个 Drawer 真实统计**：")
        if stats:
            drawer_rows = []
            for feat in range(16):
                try:
                    ds = mem.get_drawer_stats(feat)
                    drawer_rows.append({
                        'feat': f'0x{feat:01X}',
                        '类别': FEAT_NAMES.get(feat, ''),
                        '键数': ds.key_count,
                        '文档引用': ds.total_doc_refs,
                    })
                except Exception:
                    pass
            if drawer_rows:
                st.dataframe(pd.DataFrame(drawer_rows), use_container_width=True, hide_index=True)

    if layer in ["全部三层", "Layer 3: Working Memory"]:
        st.subheader("🧠 Layer 3 — Working Memory（工作记忆）")
        st.markdown("""
        - **作用**：推理期间的瞬态上下文，缓存热点记忆
        - **策略**：LRU 淘汰，最大 4096 个 HSH code
        - **命中**：查询时优先查 Working Memory，命中则直接返回
        """)
        if stats:
            col1, col2 = st.columns(2)
            col1.metric("容量上限", stats.working_memory_capacity)
            col2.metric("当前已用", stats.working_memory_used)
            if stats.working_memory_capacity > 0:
                usage_pct = stats.working_memory_used / stats.working_memory_capacity * 100
                st.progress(usage_pct / 100, text=f"使用率 {usage_pct:.1f}%")
        else:
            st.info("暂无 Working Memory 数据")

    if layer == "全部三层":
        st.subheader("🔄 三层交互流程")
        fig2, ax2 = plt.subplots(figsize=(14, 5))
        ax2.set_xlim(0, 14)
        ax2.set_ylim(0, 5)

        layers = [
            (1, 3.5, 4, 1.2, '#E74C3C', 'Layer 3\nWorking Memory\n(LRU Cache)'),
            (1, 2, 4, 1.2, '#3498DB', 'Layer 2\nArchive Index\n(16 Drawers)'),
            (1, 0.5, 4, 1.2, '#2ECC71', 'Layer 1\nToken Store\n(WAL + Buffer)'),
        ]
        for x, y, w, h, c, t in layers:
            rect = FancyBboxPatch((x, y), w, h, boxstyle="round,pad=0.1",
                                   facecolor=c, edgecolor='black', alpha=0.8)
            ax2.add_patch(rect)
            ax2.text(x + w / 2, y + h / 2, t, ha='center', va='center', color='white', fontsize=10)

        flow_x = 6
        ax2.text(flow_x, 4.5, '数据流', fontsize=12, fontweight='bold')
        steps = [
            (flow_x, 4.0, '① 插入文本 → 分词 → HSH 编码'),
            (flow_x, 3.2, '② 写入 Token Store (追加缓冲区)'),
            (flow_x, 2.4, '③ 缓冲区满 → Merge → Archive Index'),
            (flow_x, 1.6, '④ 查询 → 先查 Working Memory'),
            (flow_x, 0.8, '⑤ 未命中 → Archive Index 检索'),
        ]
        for x, y, t in steps:
            ax2.text(x, y, t, fontsize=10, bbox=dict(boxstyle='round', facecolor='wheat', alpha=0.5))
            if y > 0.8:
                ax2.annotate('', xy=(x, y - 0.15), xytext=(x, y + 0.5),
                            arrowprops=dict(arrowstyle='->', color='gray', lw=2))

        ax2.axis('off')
        st.pyplot(fig2)
