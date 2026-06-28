import streamlit as st
import pandas as pd

from pycabinet.gui import utils


def show():
    st.title("🗄️ Cabinet — 离散语义记忆检索系统")
    st.markdown("""
    **Cabinet** 是一种面向 AI Agent 的离散语义记忆检索架构，
    用 **20-bit 层次语义哈希（HSH）** 替代 768-dim 浮点向量，
    实现可解释、增量更新、纯 CPU 部署的知识检索。
    """)

    col1, col2, col3 = st.columns(3)
    with col1:
        st.metric("编码空间", "1,048,576", "20-bit")
    with col2:
        st.metric("压缩比", "~1228×", "vs 768-dim float")
    with col3:
        st.metric("检索复杂度", "O(log n)", "B-tree 前缀匹配")

    st.divider()
    st.subheader("🚀 快速操作")
    c1, c2, c3, c4 = st.columns(4)
    with c1:
        if st.button("🔢 编码可视化", use_container_width=True):
            st.session_state.page = "🔢 编码可视化"
    with c2:
        if st.button("🗂️ 记忆架构", use_container_width=True):
            st.session_state.page = "🗂️ 记忆架构"
    with c3:
        if st.button("🔍 检索路径", use_container_width=True):
            st.session_state.page = "🔍 检索路径"
    with c4:
        if st.button("⚡ 操作控制台", use_container_width=True):
            st.session_state.page = "⚡ 操作控制台"

    st.divider()
    st.subheader("📊 HSH 编码结构")
    df = pd.DataFrame({
        "位段": ["feat (4-bit)", "sim (8-bit)", "abs (8-bit)"],
        "含义": ["语义类别", "语义簇 ID", "簇内唯一标识"],
        "取值范围": ["0x0–0xF (16类)", "0x00–0xFF (256簇)", "0x00–0xFF (256桶)"],
        "示例": ["名词=0x0, 动词=0x1", "簇 #42", "桶内位置 #17"],
    })
    st.dataframe(df, use_container_width=True, hide_index=True)

    st.subheader("📐 三层记忆架构")
    arch_df = pd.DataFrame({
        "层级": ["Token Store", "Archive Index", "Working Memory"],
        "角色": ["原始文档 HSH 序列", "16 抽屉倒排索引", "推理热点缓存"],
        "存储": ["内存追加缓冲区", "B-tree + LSM", "LRU 缓存"],
        "更新策略": ["仅追加", "后台合并", "按需加载/淘汰"],
    })
    st.dataframe(arch_df, use_container_width=True, hide_index=True)
