import streamlit as st
import pandas as pd
import time
from datetime import datetime
import matplotlib.pyplot as plt
import numpy as np

# 配置中文字体（解决方块问题）
import utils

# 模拟内存状态
if 'mem_docs' not in st.session_state:
    st.session_state.mem_docs = []
if 'mem_queries' not in st.session_state:
    st.session_state.mem_queries = []
if 'wal_log' not in st.session_state:
    st.session_state.wal_log = []

def show():
    st.title("⚡ 操作控制台")
    st.markdown("交互式插入、查询、查看系统状态和 WAL 日志。")

    tab_insert, tab_query, tab_stats, tab_wal = st.tabs(["📝 插入", "🔍 查询", "📊 统计", "📋 WAL 日志"])

    with tab_insert:
        st.subheader("插入文档到记忆库")
        text = st.text_area("文档内容", "用户明天下午3点开会，准备PPT。", height=100)
        col1, col2 = st.columns([1, 4])
        with col1:
            if st.button("➕ 插入", use_container_width=True):
                with st.spinner("编码并插入..."):
                    time.sleep(0.3)  # 模拟编码耗时
                    doc_id = len(st.session_state.mem_docs) + 1
                    st.session_state.mem_docs.append({
                        'doc_id': doc_id,
                        'text': text,
                        'timestamp': datetime.now().strftime('%H:%M:%S'),
                        'chars': len(text),
                    })
                    st.session_state.wal_log.append({
                        'seq': len(st.session_state.wal_log) + 1,
                        'type': 'INSERT',
                        'doc_id': doc_id,
                        'timestamp': datetime.now().strftime('%H:%M:%S.%f')[:-3],
                        'hsh_bytes': len(text) * 3,  # 模拟
                    })
                st.success(f"✅ 已插入文档 doc_id={doc_id}")

        with col2:
            if st.button("📦 批量插入示例", use_container_width=True):
                samples = [
                    "用户喜欢听管弦乐。",
                    "明天需要准备会议材料。",
                    "下午3点有重要的项目评审。",
                    "测试数据示例文档。",
                    "这是第五条示例数据。",
                ]
                for txt in samples:
                    doc_id = len(st.session_state.mem_docs) + 1
                    st.session_state.mem_docs.append({
                        'doc_id': doc_id,
                        'text': txt,
                        'timestamp': datetime.now().strftime('%H:%M:%S'),
                        'chars': len(txt),
                    })
                    st.session_state.wal_log.append({
                        'seq': len(st.session_state.wal_log) + 1,
                        'type': 'INSERT',
                        'doc_id': doc_id,
                        'timestamp': datetime.now().strftime('%H:%M:%S.%f')[:-3],
                        'hsh_bytes': len(txt) * 3,
                    })
                st.success(f"✅ 批量插入 {len(samples)} 条文档")

        if st.session_state.mem_docs:
            st.subheader("已存储文档")
            df = pd.DataFrame(st.session_state.mem_docs)
            st.dataframe(df, use_container_width=True, hide_index=True)

    with tab_query:
        st.subheader("检索记忆")
        query = st.text_input("查询文本", "会议准备")
        top_k = st.slider("Top-K", 1, 20, 5)
        min_level = st.select_slider("最小匹配级别", options=[1, 2, 3, 4], value=1)

        if st.button("🔍 执行查询"):
            with st.spinner("检索中..."):
                time.sleep(0.2)
                # 模拟查询结果
                results = []
                for doc in st.session_state.mem_docs:
                    score = 0.0
                    level = 0
                    if query in doc['text']:
                        score = 0.95
                        level = 4
                    elif any(w in doc['text'] for w in query[:2]):
                        score = 0.5
                        level = 2
                    if level >= min_level and score > 0:
                        results.append({
                            'doc_id': doc['doc_id'],
                            'score': score,
                            'match_level': level,
                            'text': doc['text'][:50] + '...' if len(doc['text']) > 50 else doc['text'],
                        })
                results.sort(key=lambda x: x['score'], reverse=True)
                results = results[:top_k]

                st.session_state.mem_queries.append({
                    'query': query,
                    'time': datetime.now().strftime('%H:%M:%S'),
                    'results': len(results),
                })

            if results:
                st.success(f"找到 {len(results)} 条结果")
                for r in results:
                    level_colors = {4: '🔴', 3: '🟠', 2: '🟡', 1: '🟣'}
                    with st.container():
                        c1, c2, c3 = st.columns([1, 1, 4])
                        c1.markdown(f"doc_id={r['doc_id']}")
                        c2.markdown(f"{level_colors.get(r['match_level'], '⚪')} score={r['score']:.3f}")
                        c3.markdown(f"`{r['text']}`")
            else:
                st.warning("未找到匹配结果")

    with tab_stats:
        st.subheader("系统统计")
        col1, col2, col3, col4 = st.columns(4)
        col1.metric("文档总数", len(st.session_state.mem_docs))
        col2.metric("WAL 记录数", len(st.session_state.wal_log))
        col3.metric("查询次数", len(st.session_state.mem_queries))
        col4.metric("Token Store 大小", f"{len(st.session_state.mem_docs) * 3} bytes" if st.session_state.mem_docs else "0")

        if st.session_state.mem_queries:
            st.subheader("查询历史")
            qdf = pd.DataFrame(st.session_state.mem_queries)
            st.line_chart(qdf.set_index('time')['results'] if not qdf.empty else pd.DataFrame())

        st.subheader("16 个 Drawer 统计（模拟）")
        np.random.seed(42)
        drawer_stats = pd.DataFrame({
            'feat': [f'0x{i:01X}' for i in range(16)],
            'name': ['名词', '动词', '形容词', '副词', '代词', '介词', '连词', '助词',
                     '数词', '量词', '时间词', '方位词', '标点', '字符串', '常用词', '兜底'],
            'keys': np.random.randint(10, 100, 16),
            'docs': np.random.randint(50, 500, 16),
            'bytes': np.random.randint(1024, 10240, 16),
        })
        st.dataframe(drawer_stats, use_container_width=True, hide_index=True)

        import matplotlib.pyplot as plt
        fig, ax = plt.subplots(figsize=(12, 4))
        ax.bar(drawer_stats['name'], drawer_stats['keys'], color=plt.cm.tab20(np.linspace(0, 1, 16)))
        ax.set_ylabel('B-tree 键数')
        ax.set_title('各 Drawer 索引键分布')
        plt.xticks(rotation=45, ha='right')
        st.pyplot(fig)

    with tab_wal:
        st.subheader("预写日志（WAL）")
        if st.session_state.wal_log:
            wdf = pd.DataFrame(st.session_state.wal_log)
            st.dataframe(wdf, use_container_width=True, hide_index=True)
        else:
            st.info("WAL 为空，请先插入文档")

        st.markdown("""
        **WAL 格式说明：**
        ```
        [u8: type] [u64: timestamp] [u64: doc_id] [u32: hsh_len] [3×len bytes] [u32: crc32]
        ```
        - type=0x01: INSERT
        - type=0x02: DELETE
        - type=0x03: CHECKPOINT
        """)

        if st.button("🧹 清空 WAL（模拟 checkpoint）"):
            st.session_state.wal_log = []
            st.success("WAL 已清空并 checkpoint")
