@echo off
REM Installation script for auto.ui module in AutoLang stdlib (Windows)
REM
REM Usage:
REM   install-ui-module.bat [path\to\auto-lang]
REM
REM Example:
REM   install-ui-module.bat ..\..\..\auto-lang

setlocal enabledelayedexpansion

REM Default AutoLang path (relative to auto-ui)
set "AUTOLANG_PATH=%~1"
if "%AUTOLANG_PATH%"=="" set "AUTOLANG_PATH=..\..\..\auto-lang"

REM Get absolute paths
set "SCRIPT_DIR=%~dp0"
set "AUTOUI_ROOT=%SCRIPT_DIR%..\"
set "AUTOLANG_ROOT=%AUTOLANG_PATH%"

REM Convert to absolute paths
pushd "%AUTOUI_ROOT%"
set "AUTOUI_ROOT=%CD%"
popd

pushd "%AUTOLANG_ROOT%"
set "AUTOLANG_ROOT=%CD%"
popd

set "UI_SOURCE=%AUTOUI_ROOT%\stdlib\auto\ui.at"
set "UI_DEST=%AUTOLANG_ROOT%\stdlib\auto\ui.at"

echo ======================================================================
echo AutoUI Module Installation Script (Windows)
echo ======================================================================
echo.
echo AutoUI Root:  %AUTOUI_ROOT%
echo AutoLang Root: %AUTOLANG_ROOT%
echo.

REM Check if AutoLang path exists
if not exist "%AUTOLANG_ROOT%" (
    echo [ERROR] AutoLang path not found: %AUTOLANG_ROOT%
    echo.
    echo Please provide the correct path to AutoLang:
    echo   %~nx0 [path\to\auto-lang]
    echo.
    echo Example:
    echo   %~nx0 ..\..\..\auto-lang
    exit /b 1
)

REM Check if ui.at source exists
if not exist "%UI_SOURCE%" (
    echo [ERROR] ui.at not found at: %UI_SOURCE%
    exit /b 1
)

REM Create destination directory if it doesn't exist
echo Creating destination directory...
if not exist "%AUTOLANG_ROOT%\stdlib\auto" (
    mkdir "%AUTOLANG_ROOT%\stdlib\auto"
)

REM Check if ui.at already exists in AutoLang
if exist "%UI_DEST%" (
    echo [WARNING] ui.at already exists at: %UI_DEST%
    echo.
    set /p OVERWRITE="Do you want to overwrite it? (y/N): "
    if /i not "%OVERWRITE%"=="y" (
        echo Installation cancelled.
        exit /b 0
    )

    REM Backup existing file
    set BACKUP="%UI_DEST%.backup.%date:~10,4%%date:~4,2%%date:~7,2%_%time:~0,2%%time:~3,2%%time:~6,2%"
    echo Backing up existing file to: !BACKUP!
    copy "%UI_DEST%" "!BACKUP!" >nul
)

REM Ask for installation method
echo.
echo Choose installation method:
echo   1) Copy (recommended for Windows)
echo   2) Symlink (requires admin privileges)
echo.
set /p CHOICE="Enter choice (1 or 2): "
echo.

if "%CHOICE%"=="2" (
    REM Create symlink (requires admin)
    echo Creating symlink...
    mklink "%UI_DEST%" "%UI_SOURCE%" >nul 2>&1
    if errorlevel 1 (
        echo [ERROR] Failed to create symlink.
        echo.
        echo Symlinks require administrator privileges or Developer Mode enabled.
        echo Please run this script as administrator, or choose option 1 (copy).
        pause
        exit /b 1
    )
    echo [SUCCESS] Symlink created: %UI_DEST% -^> %UI_SOURCE%
) else (
    REM Copy file
    echo Copying file...
    copy "%UI_SOURCE%" "%UI_DEST%" >nul
    echo [SUCCESS] File copied to: %UI_DEST%
)

REM Verify installation
echo.
echo Verifying installation...
if exist "%UI_DEST%" (
    echo [SUCCESS] Installation successful!
    echo.
    echo Module location: %UI_DEST%
    echo.
    echo You can now use the auto.ui module in your .at files:
    echo   use auto.ui: View, Widget, App, Center, Text
    echo.
    echo Test it with:
    echo   cd %AUTOLANG_ROOT%
    echo   cargo run --bin auto-lang -- parse path\to\your\file.at
) else (
    echo [ERROR] Installation failed: %UI_DEST% not found
    exit /b 1
)

pause
