# Cabinet GUI 启动脚本（PowerShell）
# 用法：右键 → 使用 PowerShell 运行，或在终端执行：powershell -ExecutionPolicy Bypass -File 启动.ps1

$ErrorActionPreference = "Stop"

# 跳到脚本所在目录
$scriptPath = Split-Path -Parent $MyInvocation.MyCommand.Path
Set-Location $scriptPath

# 跳过 Streamlit 首次邮箱提示
$env:STREAMLIT_TELEMETRY_OPT_IN = "false"

Write-Host "=========================================" -ForegroundColor Cyan
Write-Host "   Cabinet — HSH 离散语义记忆检索系统" -ForegroundColor Cyan
Write-Host "=========================================" -ForegroundColor Cyan
Write-Host ""

# 检测 Python
try {
    $pyVer = python --version 2>&1
    Write-Host "[通过] Python 已安装: $pyVer" -ForegroundColor Green
} catch {
    Write-Host "[错误] 未找到 Python，请安装并加入 PATH" -ForegroundColor Red
    pause
    exit 1
}

# 检测依赖
try {
    python -c "import streamlit" 2>&1 | Out-Null
    Write-Host "[通过] 依赖已安装" -ForegroundColor Green
} catch {
    Write-Host "[提示] 正在安装依赖..." -ForegroundColor Yellow
    pip install -r requirements.txt
}

Write-Host ""
Write-Host "[启动] 正在启动 Cabinet GUI..." -ForegroundColor Cyan
Write-Host "[地址] http://localhost:8501" -ForegroundColor Cyan
Write-Host "[退出] 按 Ctrl + C 关闭" -ForegroundColor Gray
Write-Host ""

python -m streamlit run app.py

Write-Host ""
Write-Host "[已退出] 按任意键关闭..." -ForegroundColor Gray
$null = $Host.UI.RawUI.ReadKey("NoEcho,IncludeKeyDown")
