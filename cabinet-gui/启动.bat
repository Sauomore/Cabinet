@echo off
title Cabinet GUI 启动器
cd /d "%~dp0"

:: 跳过 Streamlit 首次邮箱提示
set STREAMLIT_TELEMETRY_OPT_IN=false

cls
echo.
echo =========================================
echo    Cabinet — HSH 离散语义记忆检索系统
echo =========================================
echo.

python -m streamlit run app.py 2>&1

echo.
echo [已退出] 按任意键关闭窗口...
pause >nul
