# -*- coding: utf-8 -*-
"""Cabinet GUI 入口：整合为 Streamlit 多页面应用。"""

import os
import sys

import streamlit as st


def _check_pycabinet():
    """检测 pycabinet Rust 扩展是否已编译。"""
    try:
        import pycabinet
        return True, None
    except ImportError as e:
        msg = (
            "### ⚠️ pycabinet 尚未编译\n\n"
            "当前 GUI 需要 Rust 核心编译后的 Python 扩展。\n\n"
            "**请按以下步骤编译：**\n\n"
            "1. 确保已安装 Rust（[https://rustup.rs](https://rustup.rs)）\n"
            "2. 确保已安装 maturin：`pip install maturin`\n"
            "3. 进入项目目录并编译：\n"
            "```bash\n"
            "cd I:/Cabinet_HSH/库文件项目/cabinet\n"
            "maturin develop\n"
            "```\n\n"
            f"**原始错误：** `{e}`"
        )
        return False, msg


def show_app():
    """渲染 Streamlit 应用主体。"""
    ok, err_msg = _check_pycabinet()
    if not ok:
        st.set_page_config(page_title="Cabinet GUI — 需要编译", page_icon="⚠️")
        st.error(err_msg, icon="⚠️")
        return

    from pycabinet.gui.pages import home, encoder_viz, memory_arch, search_path, index_browser, console

    PAGES = {
        "🏠 首页": home,
        "🔢 编码可视化": encoder_viz,
        "🗂️ 记忆架构": memory_arch,
        "🔍 检索路径": search_path,
        "📁 索引浏览器": index_browser,
        "⚡ 操作控制台": console,
    }

    st.set_page_config(
        page_title="Cabinet GUI",
        page_icon="🗄️",
        layout="wide",
        initial_sidebar_state="expanded",
    )

    st.sidebar.title("🗄️ Cabinet 导航")
    selection = st.sidebar.radio("选择页面", list(PAGES.keys()))

    page = PAGES[selection]
    page.show()

    st.sidebar.markdown("---")
    st.sidebar.caption("Cabinet v0.1.0 — HSH 离散语义记忆")


def _is_running_in_streamlit() -> bool:
    """判断当前是否已处于 streamlit 脚本运行环境中。"""
    try:
        from streamlit.runtime.scriptrunner import get_script_run_ctx
        return get_script_run_ctx() is not None
    except Exception:
        return False


def main():
    """命令行入口：通过 streamlit 运行本模块。"""
    if _is_running_in_streamlit():
        show_app()
    else:
        from streamlit.web.cli import main as st_main
        sys.argv = ["streamlit", "run", __file__, "--global.developmentMode=false"]
        st_main()


if __name__ == "__main__":
    main()
