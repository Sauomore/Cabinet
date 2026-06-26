# Cabinet GUI - 可视化操作界面

Python Streamlit 可视化桌面应用，提供 HSH 编码过程可视化、三层记忆架构图、检索路径追踪、索引结构浏览等功能。

## 安装与运行

```bash
cd cabinet-gui
pip install -r requirements.txt
streamlit run app.py
```

## 页面说明

- **🏠 首页**：项目概览、快速操作
- **🔢 编码可视化**：文本分词 → HSH 编码 → 二进制拆解
- **🗂️ 记忆架构**：Token/Archive/Working 三层架构图
- **🔍 检索路径**：查询词四级匹配流程图
- **📁 索引浏览器**：16 个 Feature Drawer 的树形展示
- **⚡ 操作控制台**：插入、查询、WAL 查看、统计
