import streamlit as st
import sys
import os

# 将 pycabinet 添加到路径（若已安装则无需）
sys.path.insert(0, os.path.join(os.path.dirname(__file__), '..', 'pycabinet'))

from pages import home, encoder_viz, memory_arch, search_path, index_browser, console

st.set_page_config(
    page_title="Cabinet GUI",
    page_icon="🗄️",
    layout="wide",
    initial_sidebar_state="expanded",
)

PAGES = {
    "🏠 首页": home,
    "🔢 编码可视化": encoder_viz,
    "🗂️ 记忆架构": memory_arch,
    "🔍 检索路径": search_path,
    "📁 索引浏览器": index_browser,
    "⚡ 操作控制台": console,
}

st.sidebar.title("🗄️ Cabinet 导航")
selection = st.sidebar.radio("选择页面", list(PAGES.keys()))

page = PAGES[selection]
page.show()

st.sidebar.markdown("---")
st.sidebar.caption("Cabinet v0.1.0 — HSH 离散语义记忆")
