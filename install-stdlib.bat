@echo off
REM ============================================================================
REM AutoUI Stdlib Installation Script
REM ============================================================================
REM This script creates symbolic links from AutoUI's stdlib to the system
REM Auto language stdlib directory, enabling the Auto parser to find ui.at
REM ============================================================================

setlocal enabledelayedexpansion

REM Configuration
set "PROJECT_DIR=%~dp0"
set "PROJECT_STD_LIB=%PROJECT_DIR%stdlib"
set "AUTO_HOME=%USERPROFILE%\.auto"
set "AUTO_LIBS=%AUTO_HOME%\libs"
set "TARGET_DIR=%AUTO_LIBS%\stdlib\auto"
set "SOURCE_FILE=%PROJECT_STD_LIB%\auto\ui.at"
set "TARGET_FILE=%TARGET_DIR%\ui.at"

echo ============================================================================
echo AutoUI Stdlib Installation
echo ============================================================================
echo.
echo Project stdlib: %SOURCE_FILE%
echo Target location: %TARGET_FILE%
echo.

REM Check if source file exists
if not exist "%SOURCE_FILE%" (
    echo [ERROR] Source file not found: %SOURCE_FILE%
    echo Please run this script from the AutoUI project root directory.
    pause
    exit /b 1
)

REM Check if running as Administrator or have Developer Mode enabled
net session >nul 2>&1
if %errorLevel% == 0 (
    echo [INFO] Running with Administrator privileges
) else (
    echo [WARN] Not running as Administrator
    echo [INFO] Attempting to create symbolic link (may require Developer Mode)...
)

REM Create target directory structure
echo.
echo [1/3] Creating directory structure...
if not exist "%AUTO_LIBS%" (
    mkdir "%AUTO_LIBS%" 2>nul
    if exist "%AUTO_LIBS%" (
        echo   Created: %AUTO_LIBS%
    ) else (
        echo   [ERROR] Failed to create: %AUTO_LIBS%
        pause
        exit /b 1
    )
)

if not exist "%TARGET_DIR%" (
    mkdir "%TARGET_DIR%" 2>nul
    if exist "%TARGET_DIR%" (
        echo   Created: %TARGET_DIR%
    ) else (
        echo   [ERROR] Failed to create: %TARGET_DIR%
        pause
        exit /b 1
    )
)

REM Remove existing symbolic link if it exists
echo.
echo [2/3] Removing old symbolic link (if exists)...
if exist "%TARGET_FILE%" (
    del "%TARGET_FILE%" 2>nul
    if exist "%TARGET_FILE%" (
        echo   [WARN] Failed to remove old link (may need admin rights)
    ) else (
        echo   Removed old link
    )
)

REM Create symbolic link
echo.
echo [3/3] Creating symbolic link...
mklink "%TARGET_FILE%" "%SOURCE_FILE%" >nul 2>&1

if %errorLevel% == 0 (
    echo.
    echo ================================================================================
    echo SUCCESS! Symbolic link created successfully.
    echo ================================================================================
    echo.
    echo Link: %TARGET_FILE%
    echo   -> %SOURCE_FILE%
    echo.
    echo You can now use 'use auto.ui: ...' in your .at files!
    echo.
) else (
    echo.
    echo ================================================================================
    echo FAILED! Could not create symbolic link.
    echo ================================================================================
    echo.
    echo This may be because:
    echo   1. You need to run this script as Administrator, OR
    echo   2. You need to enable Developer Mode in Windows Settings
    echo.
    echo To enable Developer Mode:
    echo   - Go to: Settings ^> Update & Security ^> For developers
    echo   - Enable "Developer Mode"
    echo.
    echo Alternative: Copy file instead of creating symbolic link?
    choice /C YN /M "Copy file to %AUTO_LIBS% instead"
    if errorlevel 2 (
        echo Cancelled.
        pause
        exit /b 1
    )

    echo.
    echo Copying file...
    copy /Y "%SOURCE_FILE%" "%TARGET_FILE%"
    if %errorLevel% == 0 (
        echo.
        echo ================================================================================
        echo SUCCESS! File copied (not symbolic linked).
        echo ================================================================================
        echo.
        echo File copied to: %TARGET_FILE%
        echo.
        echo NOTE: Changes to the source file will NOT be automatically synced.
        echo       Re-run this script to update the copied file.
        echo.
    ) else (
        echo [ERROR] Failed to copy file.
        pause
        exit /b 1
    )
)

pause
