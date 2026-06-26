import streamlit as st
import matplotlib.pyplot as plt
import matplotlib.patches as mpatches
from matplotlib.patches import FancyBboxPatch, FancyArrowPatch
import numpy as np

# 配置中文字体（解决方块问题）
import utils

def show():
    st.title("🗂️ 三层记忆架构可视化")
    st.markdown("Cabinet 模拟人类认知科学中的记忆分层，将记忆分为 Token、Archive、Working 三层。")

    layer = st.selectbox("选择层级深入查看", ["全部三层", "Layer 1: Token Store", "Layer 2: Archive Index", "Layer 3: Working Memory"])

    if layer in ["全部三层", "Layer 1: Token Store"]:
        st.subheader("📄 Layer 1 — Token Store（词元层）")
        st.markdown("""
        - **作用**：存储原始文档的 HSH 序列，append-only
        - **结构**：内存追加缓冲区 + WAL（预写日志）
        - **触发**：缓冲区满 1000 条 → 后台 merge 到 Archive
        """)
        if st.checkbox("显示 Token Store 模拟数据"):
            st.code("""
doc_id=1: [0x01542A, 0x142B8F, 0x0A3C12, ...]  ← "用户明天下午3点开会"
doc_id=2: [0x0F01A2, 0x223344, ...]              ← "准备PPT"
doc_id=3: [0x0E5500, 0x0A1122, ...]              ← "喜欢听管弦乐"
            """)

    if layer in ["全部三层", "Layer 2: Archive Index"]:
        st.subheader("🗄️ Layer 2 — Archive Index（档案柜索引）")
        st.markdown("""
        - **作用**：按 16 个语义类别（feat）分组的倒排索引
        - **结构**：每个 feat 一个 Drawer → 内部 B-tree 按 (sim, abs) 索引
        - **压缩**：VByte + Delta 编码 PostingList
        """)

        # 绘制 16 抽屉图
        fig, ax = plt.subplots(figsize=(12, 6))
        drawer_names = ['名词', '动词', '形容词', '副词', '代词', '介词', '连词', '助词',
                        '数词', '量词', '时间词', '方位词', '标点', '字符串', '常用词', '兜底']
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

        st.markdown("**每个 Drawer 内部结构**：")
        st.code("""
FeatureDrawer(feat=0x0):
  ├── BTreeMap<(sim,abs) → PostingList>
  │   ├── (0x15, 0x01) → [doc:1, pos:0], [doc:3, pos:2]   ← "会议"
  │   ├── (0x20, 0xAB) → [doc:2, pos:1]                   ← "PPT"
  │   └── ...
  └── VByte + Delta 压缩存储
        """)

    if layer in ["全部三层", "Layer 3: Working Memory"]:
        st.subheader("🧠 Layer 3 — Working Memory（工作记忆）")
        st.markdown("""
        - **作用**：推理期间的瞬态上下文，缓存热点记忆
        - **策略**：LRU 淘汰，最大 4096 个 HSH code
        - **命中**：查询时优先查 Working Memory，命中则直接返回
        """)
        if st.checkbox("显示 Working Memory LRU 模拟"):
            st.code("""
WorkingMemory (capacity=4096):
  [HOT] 0x01542A → doc_id=1, text="用户明天下午3点开会"
  [HOT] 0x142B8F → doc_id=1, text="准备PPT"
  [WARM] 0x0A3C12 → doc_id=3, text="喜欢听管弦乐"
  ...
  [COLD] 0xF00102 → (即将淘汰)
            """)

    if layer == "全部三层":
        st.subheader("🔄 三层交互流程")
        fig2, ax2 = plt.subplots(figsize=(14, 5))
        ax2.set_xlim(0, 14)
        ax2.set_ylim(0, 5)

        # 三层框
        layers = [
            (1, 3.5, 4, 1.2, '#E74C3C', 'Layer 3\nWorking Memory\n(LRU Cache)'),
            (1, 2, 4, 1.2, '#3498DB', 'Layer 2\nArchive Index\n(16 Drawers)'),
            (1, 0.5, 4, 1.2, '#2ECC71', 'Layer 1\nToken Store\n(WAL + Buffer)'),
        ]
        for x, y, w, h, c, t in layers:
            rect = FancyBboxPatch((x, y), w, h, boxstyle="round,pad=0.1",
                                   facecolor=c, edgecolor='black', alpha=0.8)
            ax2.add_patch(rect)
            ax2.text(x + w/2, y + h/2, t, ha='center', va='center', color='white', fontsize=10)

        # 右侧流程
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
                ax2.annotate('', xy=(x, y-0.15), xytext=(x, y+0.5),
                            arrowprops=dict(arrowstyle='->', color='gray', lw=2))

        ax2.axis('off')
        st.pyplot(fig2)
