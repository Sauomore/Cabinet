import streamlit as st
import pandas as pd
import numpy as np
import matplotlib.pyplot as plt
from matplotlib.patches import Rectangle

# 配置中文字体（解决方块问题）
import utils

def show():
    st.title("📁 索引浏览器")
    st.markdown("浏览 16 个 Feature Drawer 的内部结构，查看 B-tree 索引和 PostingList 分布。")

    feat = st.selectbox("选择 Drawer (feat)", range(16), format_func=lambda x: f"0x{x:01X} — {feat_name(x)}")

    st.subheader(f"Feature Drawer 0x{feat:01X} — {feat_name(feat)}")

    # 模拟 B-tree 数据
    np.random.seed(feat)
    n_keys = np.random.randint(20, 80)
    sims = np.random.randint(0, 256, n_keys)
    abss = np.random.randint(0, 256, n_keys)
    doc_counts = np.random.randint(1, 10, n_keys)

    df = pd.DataFrame({
        'key (sim|abs)': [f'0x{s:02X}|0x{a:02X}' for s, a in zip(sims, abss)],
        'sim': sims,
        'abs': abss,
        'doc_count': doc_counts,
        'posting_bytes': doc_counts * np.random.randint(8, 24, n_keys),
    })
    df = df.sort_values(['sim', 'abs']).reset_index(drop=True)

    col1, col2 = st.columns([3, 1])
    with col1:
        st.dataframe(df.head(50), use_container_width=True, hide_index=True)
    with col2:
        st.metric("总键数", n_keys)
        st.metric("总文档引用", int(df['doc_count'].sum()))
        st.metric("总字节", int(df['posting_bytes'].sum()))

    st.subheader("📊 键分布热力图")
    fig, ax = plt.subplots(figsize=(10, 10))
    # 将 sim/abs 映射到 16×16 网格（每格 16×16 范围）
    grid = np.zeros((16, 16))
    for _, row in df.iterrows():
        gx = row['sim'] // 16
        gy = row['abs'] // 16
        grid[gy, gx] += row['doc_count']

    im = ax.imshow(grid, cmap='YlOrRd', aspect='equal')
    ax.set_xlabel('sim // 16')
    ax.set_ylabel('abs // 16')
    ax.set_title(f'Drawer 0x{feat:01X} — 文档密度热力图 (16×16 网格)')
    plt.colorbar(im, ax=ax, label='文档引用数')

    # 标注高密区域
    for i in range(16):
        for j in range(16):
            if grid[i, j] > 0:
                ax.text(j, i, f'{int(grid[i,j])}', ha='center', va='center',
                        fontsize=6, color='white' if grid[i,j] > grid.max()*0.5 else 'black')
    st.pyplot(fig)

    st.subheader("📈 B-tree 结构可视化")
    fig2, ax2 = plt.subplots(figsize=(14, 4))
    # 将 key 按顺序排列，展示 B-tree 的叶子节点分布
    sorted_keys = df.sort_values(['sim', 'abs'])
    x_pos = np.arange(len(sorted_keys))
    colors = plt.cm.viridis(sorted_keys['doc_count'] / sorted_keys['doc_count'].max())
    ax2.bar(x_pos, sorted_keys['doc_count'], color=colors, width=1.0, edgecolor='white', linewidth=0.3)
    ax2.set_xlabel('B-tree 叶子节点顺序 (key sorted)')
    ax2.set_ylabel('文档引用数')
    ax2.set_title(f'Drawer 0x{feat:01X} — B-tree 叶子节点分布')
    ax2.set_xlim(0, len(sorted_keys))
    st.pyplot(fig2)

    st.subheader("🔍 前缀扫描演示")
    sim_filter = st.slider("选择 sim 前缀", 0, 255, 0x42)
    filtered = df[(df['sim'] == sim_filter)]
    if not filtered.empty:
        st.success(f"sim=0x{sim_filter:02X} 前缀扫描命中 {len(filtered)} 个 key，共 {filtered['doc_count'].sum()} 个文档引用")
        st.dataframe(filtered, use_container_width=True, hide_index=True)
    else:
        st.info(f"sim=0x{sim_filter:02X} 前缀扫描无命中（演示数据随机生成）")

def feat_name(feat):
    names = {0x0: '名词', 0x1: '动词', 0x2: '形容词', 0x3: '副词', 0x4: '代词',
             0x5: '介词', 0x6: '连词', 0x7: '助词', 0x8: '数词', 0x9: '量词',
             0xA: '时间词', 0xB: '方位词', 0xC: '标点', 0xD: '字符串', 0xE: '常用词', 0xF: '兜底'}
    return names.get(feat, f'0x{feat:01X}')
