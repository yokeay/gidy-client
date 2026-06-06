@echo off

echo 请选择要启动的工具：
echo 1) Claude
echo 2) Codex
set /p choice=输入选项 (1/2): 

@REM set http_proxy=http://127.0.0.1:7897
@REM set https_proxy=http://127.0.0.1:7897
set OPENAI_API_KEY=sk-iGb2AqZsy9tzPYyLWSX9LbRrwrWERMAdywvXApp78pff6DIa
set OPENAI_BASE_URL=https://api.01122002.xyz/
set ANTHROPIC_AUTH_TOKEN=sk-iGb2AqZsy9tzPYyLWSX9LbRrwrWERMAdywvXApp78pff6DIa
set ANTHROPIC_BASE_URL=https://api.01122002.xyz/

if "%choice%"=="1" (
    set ANTHROPIC_API_KEY=
    set ANTHROPIC_MODEL=glm-5.1
    echo 启动 Claude...
    claude --dangerously-skip-permissions
) else if "%choice%"=="2" (
    set OPENAI_MODEL=deepseek-v4-pro
    echo 启动 Codex...
    codex --full-auto
) else (
    echo 无效选项，请输入 1 或 2
    exit /b 1
)