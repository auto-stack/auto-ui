@echo off
REM ============================================================================
REM AutoUI Stdlib Uninstallation Script
REM ============================================================================
REM This script removes the symbolic links created by install-stdlib.bat
REM ============================================================================

setlocal enabledelayedexpansion

REM Configuration
set "AUTO_HOME=%USERPROFILE%\.auto"
set "TARGET_FILE=%AUTO_HOME%\libs\stdlib\auto\ui.at"

echo ============================================================================
echo AutoUI Stdlib Uninstallation
echo ============================================================================
echo.
echo This will remove: %TARGET_FILE%
echo.

REM Check if file exists
if not exist "%TARGET_FILE%" (
    echo [INFO] File does not exist (nothing to remove)
    pause
    exit /b 0
)

REM Confirm deletion
choice /C YN /M "Remove symbolic link"
if errorlevel 2 (
    echo Cancelled.
    pause
    exit /b 0
)

echo.
echo Removing symbolic link...
del "%TARGET_FILE%" 2>nul

if exist "%TARGET_FILE%" (
    echo.
    echo [ERROR] Failed to remove: %TARGET_FILE%
    echo You may need to run this script as Administrator.
    pause
    exit /b 1
) else (
    echo.
    echo ================================================================================
    echo SUCCESS! Symbolic link removed successfully.
    echo ================================================================================
    echo.
    echo Removed: %TARGET_FILE%
    echo.
)

pause
