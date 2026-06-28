# -*- coding: utf-8 -*-
"""操作控制台 — 通过 Rust 核心进行真实的插入、查询和统计。"""

import streamlit as st
import pandas as pd
from datetime import datetime

from pycabinet import Memory


@st.cache_resource
def get_memory():
    """缓存 Memory 实例，避免每次交互重新初始化。"""
    return Memory(
        path="./gui_memory.db",
        precision="light",
        pos_threshold=50,
        max_context=4096,
    )


@st.cache_data
def get_feat_name(feat):
    names = {
        0x0: '名词', 0x1: '动词', 0x2: '形容词', 0x3: '副词',
        0x4: '代词', 0x5: '介词', 0x6: '连词', 0x7: '助词',
        0x8: '数词', 0x9: '量词', 0xA: '时间词', 0xB: '方位词',
        0xC: '标点', 0xD: '字符串', 0xE: '常用词', 0xF: '兜底',
    }
    return names.get(feat, f'0x{feat:01X}')


def show():
    st.title("⚡ 操作控制台")
    st.markdown("通过 **Rust 核心** 进行真实的插入、查询和统计。所有数据持久化到 SQLite 文件。")

    mem = get_memory()

    tab_insert, tab_query, tab_stats, tab_bucket = st.tabs(
        ["📝 插入", "🔍 查询", "📊 统计", "📁 桶扫描"]
    )

    with tab_insert:
        st.subheader("插入文档到记忆库")
        text = st.text_area("文档内容", "用户明天下午3点开会，准备PPT。", height=100)
        col1, col2 = st.columns([1, 4])
        with col1:
            if st.button("➕ 插入", use_container_width=True):
                try:
                    doc_id = mem.insert(text)
                    st.success(f"✅ 已插入文档 doc_id={doc_id}")
                    st.rerun()
                except Exception as e:
                    st.error(f"插入失败: {e}")

        with col2:
            if st.button("📦 批量插入示例", use_container_width=True):
                samples = [
                    "用户喜欢听管弦乐。",
                    "明天需要准备会议材料。",
                    "下午3点有重要的项目评审。",
                    "测试数据示例文档。",
                    "这是第五条示例数据。",
                ]
                try:
                    ids = mem.insert_batch(samples, show_progress=True)
                    st.success(f"✅ 批量插入 {len(ids)} 条文档: {ids}")
                    st.rerun()
                except Exception as e:
                    st.error(f"批量插入失败: {e}")

        # 显示已存储的文档（从 stats 反推）
        try:
            stats = mem.get_stats()
            if stats.doc_count > 0:
                st.info(f"当前库中已有 {stats.doc_count} 条文档")
        except Exception:
            pass

    with tab_query:
        st.subheader("检索记忆")
        query = st.text_input("查询文本", "会议准备")
        top_k = st.slider("Top-K", 1, 20, 5)
        min_level = st.select_slider(
            "最小匹配级别", options=[1, 2, 3, 4], value=1,
            format_func=lambda x: {1: '1-关联', 2: '2-同类', 3: '3-同簇', 4: '4-精确'}.get(x, str(x))
        )

        if st.button("🔍 执行查询"):
            try:
                results = mem.query(query, top_k=top_k)
                # 过滤
                results = [r for r in results if r.match_level >= min_level]
                if results:
                    st.success(f"找到 {len(results)} 条结果（已过滤 match_level≥{min_level}）")
                    for r in results:
                        level_colors = {4: '🔴', 3: '🟠', 2: '🟡', 1: '🟣'}
                        level_names = {4: '精确', 3: '同簇', 2: '同类', 1: '关联'}
                        with st.container():
                            c1, c2, c3, c4 = st.columns([1, 1, 1, 4])
                            c1.markdown(f"{level_colors.get(r.match_level, '⚪')} {level_names.get(r.match_level, '?')}")
                            c2.markdown(f"**score={r.score:.3f}**")
                            c3.markdown(f"`doc_id={r.doc_id}`")
                            text = mem.decode(r)
                            c4.markdown(f"{text or '（无文本缓存）'}")
                        st.divider()
                else:
                    st.warning("未找到匹配结果")
            except Exception as e:
                st.error(f"查询失败: {e}")

    with tab_stats:
        st.subheader("系统统计")
        try:
            stats = mem.get_stats()
            col1, col2, col3, col4, col5 = st.columns(5)
            col1.metric("文档总数", stats.doc_count)
            col2.metric("下一 doc_id", stats.next_doc_id - 1)
            col3.metric("工作记忆容量", stats.working_memory_capacity)
            col4.metric("工作记忆已用", stats.working_memory_used)
            col5.metric("Token Buffer", stats.token_store_buffer_size)
            st.caption(f"精度模式: **{stats.precision}** | 存储路径: `./gui_memory.db`")
        except Exception as e:
            st.error(f"获取统计失败: {e}")

        st.subheader("16 个 Drawer 概览")
        try:
            drawer_rows = []
            for feat in range(16):
                ds = mem.get_drawer_stats(feat)
                drawer_rows.append({
                    'feat': f'0x{feat:01X}',
                    '类别': get_feat_name(feat),
                    '键数': ds.key_count,
                    '文档引用': ds.total_doc_refs,
                })
            st.dataframe(pd.DataFrame(drawer_rows), use_container_width=True, hide_index=True)
        except Exception as e:
            st.error(f"获取 Drawer 统计失败: {e}")

    with tab_bucket:
        st.subheader("桶扫描（逻辑运算）")
        c1, c2 = st.columns(2)
        with c1:
            feat = st.selectbox("feat (语义类别)", range(16), format_func=lambda x: f"0x{x:01X} — {get_feat_name(x)}")
        with c2:
            sim = st.slider("sim (语义簇)", 0, 255, 0x42)
        if st.button("🔍 扫描桶"):
            try:
                doc_ids = mem.scan_bucket(feat, sim)
                st.success(f"桶 (feat=0x{feat:01X}, sim=0x{sim:02X}) 包含 **{len(doc_ids)}** 个文档")
                if doc_ids:
                    st.write(sorted(doc_ids))
            except Exception as e:
                st.error(f"扫描失败: {e}")
