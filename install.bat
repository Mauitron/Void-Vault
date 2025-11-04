@echo off
REM Void Vault Password Manager - Windows Installation Wrapper
REM This batch file launches the PowerShell installer

echo ============================================
echo   Void Vault Password Manager - Installer
echo ============================================
echo.

REM Check if PowerShell is available
where powershell >nul 2>nul
if %ERRORLEVEL% NEQ 0 (
    echo ERROR: PowerShell not found.
    echo PowerShell is required to run the installer.
    echo.
    pause
    exit /b 1
)

REM Get the directory where this batch file is located
set "SCRIPT_DIR=%~dp0"

REM Run the PowerShell installer
echo Launching PowerShell installer...
echo.
powershell.exe -ExecutionPolicy Bypass -File "%SCRIPT_DIR%install.ps1"

REM Check if the installer completed successfully
if %ERRORLEVEL% NEQ 0 (
    echo.
    echo Installation failed or was cancelled.
    pause
    exit /b %ERRORLEVEL%
)

echo.
echo Installation completed. Check the output above for any errors.
pause
