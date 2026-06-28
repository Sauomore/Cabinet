# -*- coding: utf-8 -*-
"""编码可视化 — 通过 Rust Encoder 进行真实的 HSH 编码。"""

import streamlit as st
import pandas as pd
import matplotlib.pyplot as plt
import numpy as np

from pycabinet import Encoder


@st.cache_resource
def get_encoder():
    """缓存 Encoder 实例（无需文件路径，纯内存计算）。"""
    return Encoder()


def hsh_to_binary(hsh_u32):
    b = f'{hsh_u32:020b}'
    return (f'<span style="color:#FF6B6B">{b[:4]}</span> '
            f'<span style="color:#4ECDC4">{b[4:12]}</span> '
            f'<span style="color:#45B7D1">{b[12:]}</span>')


def show():
    st.title("🔢 HSH 编码可视化")
    st.markdown("通过 **Rust Encoder** 真实编码文本，观察分词、POS 标注和 20-bit 哈希结构。")

    encoder = get_encoder()

    text = st.text_area("输入文本", "用户明天下午3点开会，准备PPT。", height=80)
    col1, col2 = st.columns([2, 1])

    with col1:
        if st.button("🔨 编码"):
            try:
                results = encoder.encode_detail(text)
                st.session_state['enc_results'] = results
            except Exception as e:
                st.error(f"编码失败: {e}")
                return

    if 'enc_results' in st.session_state:
        results = st.session_state['enc_results']

        st.subheader("📋 分词与编码结果")
        rows = []
        for word, pos, hsh in results:
            rows.append({
                '词汇': word,
                'POS': pos,
                'feat': f"0x{hsh.feat:01X}",
                'sim': f"0x{hsh.sim:02X}",
                'abs': f"0x{hsh.abs:02X}",
                'HSH': f"0x{hsh.raw:05X}",
            })
        df = pd.DataFrame(rows)
        st.dataframe(df, use_container_width=True, hide_index=True)

        st.subheader("🧬 二进制位拆解（20-bit）")
        for word, pos, hsh in results:
            c1, c2, c3 = st.columns([1, 2, 3])
            with c1:
                st.markdown(f"**{word}**")
            with c2:
                st.markdown(f"`0x{hsh.raw:05X}`")
            with c3:
                st.markdown(hsh_to_binary(hsh.raw), unsafe_allow_html=True)

        st.subheader("📊 特征码分布")
        feat_counts = df['feat'].value_counts().sort_index()
        fig, ax = plt.subplots(figsize=(10, 4))
        colors = plt.cm.tab20(np.linspace(0, 1, len(feat_counts)))
        bars = ax.bar(feat_counts.index, feat_counts.values, color=colors, edgecolor='white')
        ax.set_xlabel('feat (4-bit)')
        ax.set_ylabel('词数')
        ax.set_title('各语义类别词汇分布')
        ax.set_xticks(range(16))
        ax.set_xticklabels([f'0x{i:01X}' for i in range(16)], fontsize=8)
        for bar in bars:
            h = bar.get_height()
            if h > 0:
                ax.text(bar.get_x() + bar.get_width() / 2, h + 0.05, str(int(h)),
                        ha='center', va='bottom', fontsize=8)
        st.pyplot(fig)

        st.subheader("🎨 HSH 空间散点图（sim × abs）")
        feat_names = {
            0x0: '名词', 0x1: '动词', 0x2: '形容词', 0x3: '副词', 0x4: '代词',
            0x5: '介词', 0x6: '连词', 0x7: '助词', 0x8: '数词', 0x9: '量词',
            0xA: '时间词', 0xB: '方位词', 0xC: '标点', 0xD: '字符串', 0xE: '常用词', 0xF: '兜底'
        }
        fig2, ax2 = plt.subplots(figsize=(8, 8))
        # 按 feat 分组绘制
        for _, row in df.iterrows():
            feat_val = int(row['feat'], 16)
            ax2.scatter(int(row['sim'], 16), int(row['abs'], 16), alpha=0.7, s=150,
                        edgecolors='white', linewidths=1)
            ax2.annotate(row['词汇'], (int(row['sim'], 16), int(row['abs'], 16)),
                        fontsize=8, ha='center', va='bottom')
        ax2.set_xlim(0, 255)
        ax2.set_ylim(0, 255)
        ax2.set_xlabel('sim (8-bit)')
        ax2.set_ylabel('abs (8-bit)')
        ax2.set_title('HSH 编码空间分布（256 × 256）')
        ax2.grid(True, alpha=0.3)
        st.pyplot(fig2)
