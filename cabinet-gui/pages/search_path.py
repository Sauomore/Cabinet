import streamlit as st
import matplotlib.pyplot as plt
import matplotlib.patches as mpatches
from matplotlib.patches import FancyBboxPatch, FancyArrowPatch
import numpy as np

# 配置中文字体（解决方块问题）
import utils

def show():
    st.title("🔍 检索路径可视化")
    st.markdown("追踪一次查询如何从 **HSH 编码** → **四级匹配** → **文档聚合排序** 的完整过程。")

    query = st.text_input("查询文本", "会议准备")
    if st.button("🚀 执行检索"):
        with st.spinner("模拟检索路径..."):
            show_search_flow(query)

def show_search_flow(query):
    # 模拟查询词的分词和 HSH
    query_words = query  # 简化
    query_hsh = [0x01542A, 0x142B8F]  # 模拟 "会议" 和 "准备" 的 HSH

    st.subheader("1️⃣ 查询词 HSH 编码")
    cols = st.columns(len(query_hsh))
    for i, hsh in enumerate(query_hsh):
        with cols[i]:
            feat = (hsh >> 16) & 0x0F
            sim = (hsh >> 8) & 0xFF
            abs = hsh & 0xFF
            st.metric(f"词 {i+1}", f"0x{hsh:05X}")
            st.caption(f"feat=0x{feat:01X} | sim=0x{sim:02X} | abs=0x{abs:02X}")

    st.subheader("2️⃣ 四级匹配流程")
    fig, ax = plt.subplots(figsize=(14, 8))
    ax.set_xlim(0, 14)
    ax.set_ylim(0, 10)

    # 查询词节点
    y_query = 9
    for i, hsh in enumerate(query_hsh):
        x = 2 + i * 5
        rect = FancyBboxPatch((x, y_query-0.3), 2, 0.6, boxstyle="round,pad=0.05",
                               facecolor='#3498DB', edgecolor='black')
        ax.add_patch(rect)
        ax.text(x+1, y_query, f'Q{i+1}: 0x{hsh:05X}', ha='center', va='center', color='white', fontsize=9)

    # 四级匹配层
    levels = [
        (8.0, 'Level 4', '精确匹配', '#E74C3C', 'exact_query(sim, abs) → 命中唯一绝对码'),
        (6.5, 'Level 3', '同簇匹配', '#E67E22', 'sim_prefix_scan(sim) → 同语义簇'),
        (5.0, 'Level 2', '同类匹配', '#F1C40F', '整个 Drawer 扫描 → 同词性类别'),
        (3.5, 'Level 1', '关联类别', '#9B59B6', 'Router.get_related(feat) → 跨类别关联'),
    ]

    for y, label, name, color, desc in levels:
        rect = FancyBboxPatch((1, y-0.35), 12, 0.7, boxstyle="round,pad=0.05",
                               facecolor=color, edgecolor='black', alpha=0.7)
        ax.add_patch(rect)
        ax.text(2, y, f'{label}: {name}', ha='left', va='center', color='white', fontsize=10, fontweight='bold')
        ax.text(7, y, desc, ha='left', va='center', color='white', fontsize=9)

    # 箭头
    for i, hsh in enumerate(query_hsh):
        x = 3 + i * 5
        ax.annotate('', xy=(x, 7.65), xytext=(x, 8.7),
                    arrowprops=dict(arrowstyle='->', color='gray', lw=1.5))
        for y in [7.0, 5.5, 4.0]:
            ax.annotate('', xy=(x, y-0.35), xytext=(x, y+0.35),
                        arrowprops=dict(arrowstyle='->', color='gray', lw=1, ls='--'))

    # 聚合与排序
    y_agg = 2.0
    rect = FancyBboxPatch((3, y_agg-0.4), 8, 0.8, boxstyle="round,pad=0.05",
                           facecolor='#2ECC71', edgecolor='black')
    ax.add_patch(rect)
    ax.text(7, y_agg, '文档聚合排序\nscore(d, Q) = Σ max(ω · pos_prox)', 
            ha='center', va='center', color='white', fontsize=10, fontweight='bold')

    ax.annotate('', xy=(7, 1.6), xytext=(7, 3.15),
                arrowprops=dict(arrowstyle='->', color='black', lw=2))

    # 结果
    y_res = 1.0
    rect = FancyBboxPatch((3, y_res-0.3), 8, 0.6, boxstyle="round,pad=0.05",
                           facecolor='#1ABC9C', edgecolor='black')
    ax.add_patch(rect)
    ax.text(7, y_res, 'Top-K 结果返回', ha='center', va='center', color='white', fontsize=10)

    ax.axis('off')
    ax.set_title(f'检索路径："{query}"', fontsize=14, pad=20)
    st.pyplot(fig)

    st.subheader("3️⃣ 模拟检索结果")
    results = [
        {'doc_id': 1, 'match_level': 4, 'score': 0.95, 'text': '用户明天下午3点开会，准备PPT。'},
        {'doc_id': 5, 'match_level': 3, 'score': 0.72, 'text': '明天的会议需要提前准备。'},
        {'doc_id': 2, 'match_level': 2, 'score': 0.45, 'text': '用户喜欢听管弦乐。'},
    ]
    for r in results:
        level_colors = {4: '🔴', 3: '🟠', 2: '🟡', 1: '🟣'}
        with st.container():
            c1, c2, c3 = st.columns([1, 2, 4])
            with c1:
                st.markdown(f"{level_colors.get(r['match_level'], '⚪')} Level {r['match_level']}")
            with c2:
                st.markdown(f"**score: {r['score']:.3f}**")
            with c3:
                st.caption(f"doc_id={r['doc_id']} | {r['text']}")
            st.divider()

    st.subheader("4️⃣ 匹配级别说明")
    st.markdown("""
    | 级别 | 名称 | 匹配条件 | 权重 | 说明 |
    |------|------|----------|------|------|
    | 4 | 精确匹配 | feat + sim + abs 完全一致 | 1.0 | 同一词汇或同簇完美哈希命中 |
    | 3 | 同簇匹配 | feat + sim 相同，abs 不同 | 0.7 | 同一语义簇内的不同词 |
    | 2 | 同类匹配 | feat 相同，sim 不同 | 0.4 | 同一词性类别但不同语义簇 |
    | 1 | 关联类别 | 通过 Router 关联的 feat | 0.2×weight | 跨类别语义关联（如名词↔动词） |
    """)
