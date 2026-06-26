import streamlit as st
import pandas as pd
import matplotlib.pyplot as plt
import matplotlib.patches as mpatches
import numpy as np

# 配置中文字体（解决方块问题）
import utils

# 模拟 HSH 编码（纯 Python 演示，无需加载 Rust 库）
def mock_jieba_cut(text):
    """简化分词：按空格/标点粗略分词"""
    import re
    words = re.findall(r'[\u4e00-\u9fff]+|[a-zA-Z]+|[0-9]+|[，。；：？！、]', text)
    return words

def mock_pos(word):
    """简化 POS 标注"""
    if word in '，。；：？！、':
        return 'w', '标点'
    if word.isdigit():
        return 'm', '数词'
    if word.encode('utf-8').isalpha():
        return 'x', '字符串'
    # 简化：前几个字是名词，后几个是动词... 仅演示
    if len(word) >= 2 and word[-1] in '会备':
        return 'v', '动词'
    return 'n', '名词'

def feat_from_pos(pos):
    mapping = {'n': 0x0, 'v': 0x1, 'a': 0x2, 'd': 0x3, 'r': 0x4,
               'p': 0x5, 'c': 0x6, 'u': 0x7, 'm': 0x8, 'q': 0x9,
               't': 0xA, 'f': 0xB, 'w': 0xC, 'x': 0xD, 'nz': 0xE}
    return mapping.get(pos, 0xF)

def mock_sim(word, feat):
    import hashlib
    h = int(hashlib.md5(word.encode()).hexdigest(), 16)
    return (h + feat * 31) % 256

def mock_abs(word, feat, sim):
    import hashlib
    h = int(hashlib.md5((word + str(sim)).encode()).hexdigest(), 16)
    return (h ^ (feat * 17)) % 256

def encode_hsh(text):
    words = mock_jieba_cut(text)
    results = []
    for w in words:
        pos, pos_name = mock_pos(w)
        feat = feat_from_pos(pos)
        sim = mock_sim(w, feat)
        abs = mock_abs(w, feat, sim)
        hsh = (feat << 16) | (sim << 8) | abs
        results.append({
            'word': w, 'pos': pos, 'pos_name': pos_name,
            'feat': feat, 'sim': sim, 'abs': abs,
            'hsh_hex': f'0x{hsh:05X}', 'hsh_u32': hsh,
        })
    return results

def hsh_to_binary(hsh_u32):
    b = f'{hsh_u32:020b}'
    return f'<span style="color:#FF6B6B">{b[:4]}</span> ' \
           f'<span style="color:#4ECDC4">{b[4:12]}</span> ' \
           f'<span style="color:#45B7D1">{b[12:]}</span>'

def show():
    st.title("🔢 HSH 编码可视化")
    st.markdown("观察文本如何被分词、标注词性、映射为 20-bit 层次语义哈希。")

    text = st.text_area("输入文本", "用户明天下午3点开会，准备PPT。", height=80)
    col1, col2 = st.columns([2, 1])

    with col1:
        if st.button("🔨 编码"):
            results = encode_hsh(text)
            st.session_state['enc_results'] = results

    if 'enc_results' in st.session_state:
        results = st.session_state['enc_results']

        st.subheader("📋 分词与编码结果")
        df = pd.DataFrame(results)
        df['feat_hex'] = df['feat'].apply(lambda x: f'0x{x:01X}')
        df['sim_hex'] = df['sim'].apply(lambda x: f'0x{x:02X}')
        df['abs_hex'] = df['abs'].apply(lambda x: f'0x{x:02X}')
        display = df[['word', 'pos_name', 'feat_hex', 'sim_hex', 'abs_hex', 'hsh_hex']]
        display.columns = ['词汇', '词性', '特征码', '相似码', '绝对码', 'HSH']
        st.dataframe(display, use_container_width=True, hide_index=True)

        st.subheader("🧬 二进制位拆解（20-bit）")
        for r in results:
            c1, c2, c3 = st.columns([1, 2, 3])
            with c1:
                st.markdown(f"**{r['word']}**")
            with c2:
                st.markdown(f"`{r['hsh_hex']}`")
            with c3:
                st.markdown(hsh_to_binary(r['hsh_u32']), unsafe_allow_html=True)

        st.subheader("📊 特征码分布")
        fig, ax = plt.subplots(figsize=(10, 4))
        feat_counts = df['feat'].value_counts().sort_index()
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
                ax.text(bar.get_x() + bar.get_width()/2, h + 0.05, str(int(h)),
                        ha='center', va='bottom', fontsize=8)
        st.pyplot(fig)

        st.subheader("🎨 HSH 空间散点图（sim × abs）")
        fig2, ax2 = plt.subplots(figsize=(8, 8))
        feat_names = {0x0: '名词', 0x1: '动词', 0x2: '形容词', 0x3: '副词', 0x4: '代词',
                      0x5: '介词', 0x6: '连词', 0x7: '助词', 0x8: '数词', 0x9: '量词',
                      0xA: '时间词', 0xB: '方位词', 0xC: '标点', 0xD: '字符串', 0xE: '常用词', 0xF: '兜底'}
        for feat in sorted(df['feat'].unique()):
            subset = df[df['feat'] == feat]
            ax2.scatter(subset['sim'], subset['abs'], alpha=0.7, s=100,
                        label=f'{feat_names.get(feat, f"0x{feat:01X}")}', edgecolors='white')
        ax2.set_xlim(0, 255)
        ax2.set_ylim(0, 255)
        ax2.set_xlabel('sim (8-bit)')
        ax2.set_ylabel('abs (8-bit)')
        ax2.set_title('HSH 编码空间分布（256 × 256）')
        ax2.legend(loc='upper right', fontsize=8, ncol=2)
        ax2.grid(True, alpha=0.3)
        st.pyplot(fig2)
