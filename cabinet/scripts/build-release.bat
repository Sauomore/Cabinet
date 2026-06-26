@echo off
title Cabinet Release Builder
chcp 65001 >nul
cd /d "%~dp0\.."

cls
echo =========================================
echo    Cabinet Release Builder
echo =========================================
echo.

:: 检查 Python
python --version >nul 2>&1
if errorlevel 1 (
    echo [错误] 未找到 Python
    pause
    exit /b 1
)

:: 检查 Rust
cargo --version >nul 2>&1
if errorlevel 1 (
    echo [错误] 未找到 Rust 工具链
    echo 请访问 https://rustup.rs/ 安装
    pause
    exit /b 1
)

echo [通过] Python: & python --version
echo [通过] Rust: & cargo --version
echo.

:: 安装构建工具
echo [步骤 1/3] 安装构建工具...
pip install maturin cibuildwheel 2>&1 | findstr /v "already satisfied"

:: 构建 wheel
echo.
echo [步骤 2/3] 构建 Windows wheel...
cd pycabinet
cibuildwheel --platform windows --output-dir wheelhouse

:: 测试安装
echo.
echo [步骤 3/3] 测试安装...
for %%f in (wheelhouse\*.whl) do (
    pip install --force-reinstall "%%f"
    python -c "import pycabinet; m = pycabinet.Memory('./test_release.db'); m.insert('test'); print('OK')"
    goto :done
)
:done

cd ..
echo.
echo [完成] 构建产物: %cd%\pycabinet\wheelhouse\
explorer "pycabinet\wheelhouse"
pause
