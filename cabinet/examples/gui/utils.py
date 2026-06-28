# -*- coding: utf-8 -*-
"""Cabinet GUI 公共工具：字体配置等"""
import warnings

# 解决 matplotlib 中文显示方块问题
def setup_fonts():
    """设置 matplotlib 中文字体，按优先级尝试（失败时静默跳过）"""
    try:
        import matplotlib.pyplot as plt
        import matplotlib.font_manager as fm
    except ImportError:
        warnings.warn("[utils] matplotlib 未安装，跳过字体设置")
        return

    try:
        # 候选字体列表（Windows / macOS / Linux）
        candidates = [
            'Microsoft YaHei',      # 微软雅黑（Windows 最常用）
            'SimHei',               # 黑体（Windows 老系统）
            'SimSun',               # 宋体
            'WenQuanYi Micro Hei',  # Linux 文泉驿
            'Noto Sans CJK SC',     # Linux Noto
            'PingFang SC',          # macOS 苹方
            'Heiti SC',             # macOS 黑体
            'Arial Unicode MS',     # 通用
        ]
        
        available = {f.name for f in fm.fontManager.ttflist}
        
        selected = None
        for font in candidates:
            if font in available:
                selected = font
                break
        
        if selected:
            plt.rcParams['font.sans-serif'] = [selected, 'DejaVu Sans']
            plt.rcParams['axes.unicode_minus'] = False
            print(f"[utils] 已设置中文字体: {selected}")
        else:
            # 回退：尝试查找任何包含 CJK/Hei/YaHei/Sim 的字体
            cjk_fonts = [f.name for f in fm.fontManager.ttflist 
                         if 'CJK' in f.name or 'Hei' in f.name or 'YaHei' in f.name or 'Sim' in f.name]
            if cjk_fonts:
                plt.rcParams['font.sans-serif'] = [cjk_fonts[0], 'DejaVu Sans']
                plt.rcParams['axes.unicode_minus'] = False
                print(f"[utils] 已设置回退中文字体: {cjk_fonts[0]}")
            else:
                warnings.warn("[utils] 未找到中文字体，图表可能显示方块")
    except Exception as e:
        warnings.warn(f"[utils] 字体设置失败: {e}")

# 应用字体配置
setup_fonts()
