# -*- coding: utf-8 -*-
"""检索路径可视化 — 通过 Rust Memory 进行真实查询，追踪四级匹配。"""

import streamlit as st
import matplotlib.pyplot as plt
from matplotlib.patches import FancyBboxPatch

from pycabinet import Memory


@st.cache_resource
def get_memory():
    return Memory(
        path="./gui_memory.db",
        precision="light",
        pos_threshold=50,
        max_context=4096,
    )


def show():
    st.title("🔍 检索路径可视化")
    st.markdown("输入查询文本，通过 **Rust Memory** 执行真实检索，追踪四级匹配路径。")

    mem = get_memory()

    query = st.text_input("查询文本", "会议准备")
    top_k = st.slider("Top-K", 1, 20, 5)

    if st.button("🚀 执行检索"):
        try:
            # 先获取编码详情（展示查询词的 HSH）
            enc_details = mem.encode_detail(query)
            show_search_flow(mem, query, enc_details, top_k)
        except Exception as e:
            st.error(f"检索失败: {e}")


def show_search_flow(mem, query, enc_details, top_k):
    query_hsh = [(word, pos, hsh.raw) for word, pos, hsh in enc_details]

    st.subheader("1️⃣ 查询词 HSH 编码")
    if query_hsh:
        cols = st.columns(len(query_hsh))
        for i, (word, pos, hsh) in enumerate(query_hsh):
            feat = (hsh >> 16) & 0x0F
            sim = (hsh >> 8) & 0xFF
            abs = hsh & 0xFF
            with cols[i]:
                st.markdown(f"**{word}**")
                st.metric("HSH", f"0x{hsh:05X}")
                st.caption(f"feat=0x{feat:01X} | sim=0x{sim:02X} | abs=0x{abs:02X}")
                st.caption(f"POS={pos}")
    else:
        st.warning("查询词无法编码（空结果）")

    # 执行真实查询
    results = mem.query(query, top_k=top_k)

    st.subheader("2️⃣ 四级匹配流程（理论模型）")
    fig, ax = plt.subplots(figsize=(14, 8))
    ax.set_xlim(0, 14)
    ax.set_ylim(0, 10)

    y_query = 9
    for i, (_, _, hsh) in enumerate(query_hsh):
        x = 2 + i * 5
        rect = FancyBboxPatch((x, y_query - 0.3), 2, 0.6, boxstyle="round,pad=0.05",
                               facecolor='#3498DB', edgecolor='black')
        ax.add_patch(rect)
        ax.text(x + 1, y_query, f'Q{i + 1}: 0x{hsh:05X}',
                ha='center', va='center', color='white', fontsize=9)

    levels = [
        (8.0, 'Level 4', '精确匹配', '#E74C3C', 'exact_query(sim, abs) → 命中唯一绝对码'),
        (6.5, 'Level 3', '同簇匹配', '#E67E22', 'sim_prefix_scan(sim) → 同语义簇'),
        (5.0, 'Level 2', '同类匹配', '#F1C40F', '整个 Drawer 扫描 → 同词性类别'),
        (3.5, 'Level 1', '关联类别', '#9B59B6', 'Router.get_related(feat) → 跨类别关联'),
    ]

    for y, label, name, color, desc in levels:
        rect = FancyBboxPatch((1, y - 0.35), 12, 0.7, boxstyle="round,pad=0.05",
                               facecolor=color, edgecolor='black', alpha=0.7)
        ax.add_patch(rect)
        ax.text(2, y, f'{label}: {name}', ha='left', va='center', color='white', fontsize=10, fontweight='bold')
        ax.text(7, y, desc, ha='left', va='center', color='white', fontsize=9)

    for i in range(len(query_hsh)):
        x = 3 + i * 5
        ax.annotate('', xy=(x, 7.65), xytext=(x, 8.7),
                    arrowprops=dict(arrowstyle='->', color='gray', lw=1.5))
        for y in [7.0, 5.5, 4.0]:
            ax.annotate('', xy=(x, y - 0.35), xytext=(x, y + 0.35),
                        arrowprops=dict(arrowstyle='->', color='gray', lw=1, ls='--'))

    y_agg = 2.0
    rect = FancyBboxPatch((3, y_agg - 0.4), 8, 0.8, boxstyle="round,pad=0.05",
                           facecolor='#2ECC71', edgecolor='black')
    ax.add_patch(rect)
    ax.text(7, y_agg, '文档聚合排序\nscore(d, Q) = Σ max(ω · pos_prox)',
            ha='center', va='center', color='white', fontsize=10, fontweight='bold')

    ax.annotate('', xy=(7, 1.6), xytext=(7, 3.15),
                arrowprops=dict(arrowstyle='->', color='black', lw=2))

    y_res = 1.0
    rect = FancyBboxPatch((3, y_res - 0.3), 8, 0.6, boxstyle="round,pad=0.05",
                           facecolor='#1ABC9C', edgecolor='black')
    ax.add_patch(rect)
    ax.text(7, y_res, 'Top-K 结果返回', ha='center', va='center', color='white', fontsize=10)

    ax.axis('off')
    ax.set_title(f'检索路径："{query}"', fontsize=14, pad=20)
    st.pyplot(fig)

    st.subheader("3️⃣ 真实检索结果")
    if results:
        level_colors = {4: '🔴', 3: '🟠', 2: '🟡', 1: '🟣'}
        level_names = {4: '精确', 3: '同簇', 2: '同类', 1: '关联'}
        for r in results:
            with st.container():
                c1, c2, c3, c4 = st.columns([1, 1, 1, 4])
                c1.markdown(f"{level_colors.get(r.match_level, '⚪')} {level_names.get(r.match_level, '?')}")
                c2.markdown(f"**score={r.score:.3f}**")
                c3.markdown(f"`doc_id={r.doc_id}`")
                text = mem.decode(r)
                c4.markdown(f"{text or '（无文本缓存）'}")
            st.divider()
    else:
        st.info("未找到匹配结果。请先插入一些文档。")

    st.subheader("4️⃣ 匹配级别说明")
    st.markdown("""
    | 级别 | 名称 | 匹配条件 | 权重 | 说明 |
    |------|------|----------|------|------|
    | 4 | 精确匹配 | feat + sim + abs 完全一致 | 1.0 | 同一词汇或同簇完美哈希命中 |
    | 3 | 同簇匹配 | feat + sim 相同，abs 不同 | 0.7 | 同一语义簇内的不同词 |
    | 2 | 同类匹配 | feat 相同，sim 不同 | 0.4 | 同一词性类别但不同语义簇 |
    | 1 | 关联类别 | 通过 Router 关联的 feat | 0.2×weight | 跨类别语义关联（如名词↔动词） |
    """)
