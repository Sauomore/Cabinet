# -*- coding: utf-8 -*-
"""索引浏览器 — 通过 Rust Memory 获取真实的 Drawer 统计和桶数据。"""

import streamlit as st
import pandas as pd
import numpy as np
import matplotlib.pyplot as plt

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
    st.title("📁 索引浏览器")
    st.markdown("浏览 16 个 Feature Drawer 的真实内部结构，查看 B-tree 索引和 PostingList 分布。")

    mem = get_memory()

    feat = st.selectbox(
        "选择 Drawer (feat)", range(16),
        format_func=lambda x: f"0x{x:01X} — {FEAT_NAMES.get(x, '未知')}"
    )

    st.subheader(f"Feature Drawer 0x{feat:01X} — {FEAT_NAMES.get(feat, '未知')}")

    try:
        ds = mem.get_drawer_stats(feat)
    except Exception as e:
        st.error(f"获取 Drawer 统计失败: {e}")
        return

    col1, col2, col3 = st.columns(3)
    col1.metric("总键数", ds.key_count)
    col2.metric("总文档引用", ds.total_doc_refs)
    col3.metric("键明细数", len(ds.keys))

    if not ds.keys:
        st.info("该 Drawer 暂无数据。请先插入一些文档。")
        return

    # 构建 DataFrame
    rows = []
    for key, doc_count, posting_bytes in ds.keys:
        sim = (key >> 8) & 0xFF
        abs = key & 0xFF
        rows.append({
            'key': f'0x{key:04X}',
            'sim': f'0x{sim:02X}',
            'abs': f'0x{abs:02X}',
            'doc_count': doc_count,
            'posting_bytes': posting_bytes,
        })
    df = pd.DataFrame(rows)
    df = df.sort_values(['sim', 'abs']).reset_index(drop=True)

    st.subheader("📋 B-tree 键列表")
    st.dataframe(df, use_container_width=True, hide_index=True)

    st.subheader("📊 键分布热力图")
    fig, ax = plt.subplots(figsize=(10, 10))
    grid = np.zeros((16, 16))
    for _, row in df.iterrows():
        sim_val = int(row['sim'], 16)
        abs_val = int(row['abs'], 16)
        gx = sim_val // 16
        gy = abs_val // 16
        grid[gy, gx] += row['doc_count']

    im = ax.imshow(grid, cmap='YlOrRd', aspect='equal')
    ax.set_xlabel('sim // 16')
    ax.set_ylabel('abs // 16')
    ax.set_title(f'Drawer 0x{feat:01X} — 文档密度热力图 (16×16 网格)')
    plt.colorbar(im, ax=ax, label='文档引用数')

    for i in range(16):
        for j in range(16):
            if grid[i, j] > 0:
                ax.text(j, i, f'{int(grid[i,j])}', ha='center', va='center',
                        fontsize=6, color='white' if grid[i, j] > grid.max() * 0.5 else 'black')
    st.pyplot(fig)

    st.subheader("📈 B-tree 叶子节点分布")
    fig2, ax2 = plt.subplots(figsize=(14, 4))
    sorted_df = df.sort_values(['sim', 'abs']).reset_index(drop=True)
    x_pos = np.arange(len(sorted_df))
    colors = plt.cm.viridis(sorted_df['doc_count'] / sorted_df['doc_count'].max())
    ax2.bar(x_pos, sorted_df['doc_count'], color=colors, width=1.0, edgecolor='white', linewidth=0.3)
    ax2.set_xlabel('B-tree 叶子节点顺序 (key sorted)')
    ax2.set_ylabel('文档引用数')
    ax2.set_title(f'Drawer 0x{feat:01X} — B-tree 叶子节点分布')
    ax2.set_xlim(0, len(sorted_df))
    st.pyplot(fig2)

    st.subheader("🔍 前缀扫描演示")
    sim_filter = st.slider("选择 sim 前缀", 0, 255, 0x42)
    # sim 是字符串形式 '0x42'，需要比较整数
    filtered = df[df['sim'] == f'0x{sim_filter:02X}']
    if not filtered.empty:
        st.success(
            f"sim=0x{sim_filter:02X} 前缀扫描命中 {len(filtered)} 个 key，"
            f"共 {int(filtered['doc_count'].sum())} 个文档引用"
        )
        st.dataframe(filtered, use_container_width=True, hide_index=True)
    else:
        st.info(f"sim=0x{sim_filter:02X} 前缀扫描无命中")
