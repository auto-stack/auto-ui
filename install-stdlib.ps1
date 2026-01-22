# ============================================================================
# AutoUI Stdlib Installation Script (PowerShell)
# ============================================================================
# This script creates symbolic links from AutoUI's stdlib to the system
# Auto language stdlib directory, enabling the Auto parser to find ui.at
# ============================================================================

# Configuration
$ProjectRoot = $PSScriptRoot
$ProjectStdLib = Join-Path $ProjectRoot "stdlib"
$SourceFile = Join-Path $ProjectStdLib "auto\ui.at"
$AutoHome = Join-Path $env:USERPROFILE ".auto"
$AutoLibs = Join-Path $AutoHome "libs"
$TargetDir = Join-Path $AutoLibs "stdlib\auto"
$TargetFile = Join-Path $TargetDir "ui.at"

function Write-Header {
    Write-Host "================================================================================" -ForegroundColor Cyan
    Write-Host "AutoUI Stdlib Installation (PowerShell)" -ForegroundColor Cyan
    Write-Host "================================================================================" -ForegroundColor Cyan
    Write-Host ""
    Write-Host "Project stdlib: $SourceFile" -ForegroundColor White
    Write-Host "Target location: $TargetFile" -ForegroundColor White
    Write-Host ""
}

function Test-Administrator {
    $currentUser = [Security.Principal.WindowsIdentity]::GetCurrent()
    $principal = New-Object Security.Principal.WindowsPrincipal($currentUser)
    return $principal.IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)
}

function Enable-DeveloperMode {
    Write-Host ""
    Write-Host "[INFO] To create symbolic links on Windows, you need either:" -ForegroundColor Yellow
    Write-Host "  1. Administrator privileges, OR" -ForegroundColor White
    Write-Host "  2. Developer Mode enabled" -ForegroundColor White
    Write-Host ""
    Write-Host "To enable Developer Mode:" -ForegroundColor Cyan
    Write-Host "  1. Go to: Settings > Update & Security > For developers" -ForegroundColor White
    Write-Host "  2. Enable 'Developer Mode'" -ForegroundColor White
    Write-Host ""
}

function Install-Stdlib {
    Write-Header

    # Check if source file exists
    if (-not (Test-Path $SourceFile)) {
        Write-Host "[ERROR] Source file not found: $SourceFile" -ForegroundColor Red
        Write-Host "Please run this script from the AutoUI project root directory." -ForegroundColor Red
        Read-Host "Press Enter to exit"
        exit 1
    }

    # Check admin privileges
    $isAdmin = Test-Administrator
    if ($isAdmin) {
        Write-Host "[INFO] Running with Administrator privileges" -ForegroundColor Green
    } else {
        Write-Host "[WARN] Not running as Administrator" -ForegroundColor Yellow
    }

    # Step 1: Create directory structure
    Write-Host ""
    Write-Host "[1/3] Creating directory structure..." -ForegroundColor Cyan

    if (-not (Test-Path $AutoLibs)) {
        New-Item -ItemType Directory -Path $AutoLibs -Force | Out-Null
        if (Test-Path $AutoLibs) {
            Write-Host "  Created: $AutoLibs" -ForegroundColor Green
        }
    }

    if (-not (Test-Path $TargetDir)) {
        New-Item -ItemType Directory -Path $TargetDir -Force | Out-Null
        if (Test-Path $TargetDir) {
            Write-Host "  Created: $TargetDir" -ForegroundColor Green
        }
    }

    # Step 2: Remove old symbolic link if exists
    Write-Host ""
    Write-Host "[2/3] Removing old symbolic link (if exists)..." -ForegroundColor Cyan

    if (Test-Path $TargetFile) {
        try {
            Remove-Item $TargetFile -Force -ErrorAction Stop
            Write-Host "  Removed old link" -ForegroundColor Green
        } catch {
            Write-Host "  [WARN] Failed to remove old link: $_" -ForegroundColor Yellow
        }
    }

    # Step 3: Create symbolic link
    Write-Host ""
    Write-Host "[3/3] Creating symbolic link..." -ForegroundColor Cyan

    try {
        New-Item -ItemType SymbolicLink -Path $TargetFile -Target $SourceFile -ErrorAction Stop | Out-Null

        Write-Host ""
        Write-Host "================================================================================" -ForegroundColor Green
        Write-Host "SUCCESS! Symbolic link created successfully." -ForegroundColor Green
        Write-Host "================================================================================" -ForegroundColor Green
        Write-Host ""
        Write-Host "Link: $TargetFile" -ForegroundColor White
        Write-Host "  -> $SourceFile" -ForegroundColor White
        Write-Host ""
        Write-Host "You can now use 'use auto.ui: ...' in your .at files!" -ForegroundColor Green
        Write-Host ""

    } catch {
        Write-Host ""
        Write-Host "================================================================================" -ForegroundColor Red
        Write-Host "FAILED! Could not create symbolic link." -ForegroundColor Red
        Write-Host "================================================================================" -ForegroundColor Red
        Write-Host ""
        Write-Host "Error: $($_.Exception.Message)" -ForegroundColor Red
        Write-Host ""

        Enable-DeveloperMode

        # Offer to copy instead
        $response = Read-Host "Copy file instead of creating symbolic link? (Y/N)"
        if ($response -eq 'Y' -or $response -eq 'y') {
            Write-Host ""
            Write-Host "Copying file..." -ForegroundColor Cyan
            Copy-Item -Path $SourceFile -Destination $TargetFile -Force

            if (Test-Path $TargetFile) {
                Write-Host ""
                Write-Host "================================================================================" -ForegroundColor Green
                Write-Host "SUCCESS! File copied (not symbolic linked)." -ForegroundColor Green
                Write-Host "================================================================================" -ForegroundColor Green
                Write-Host ""
                Write-Host "File copied to: $TargetFile" -ForegroundColor White
                Write-Host ""
                Write-Host "NOTE: Changes to the source file will NOT be automatically synced." -ForegroundColor Yellow
                Write-Host "      Re-run this script to update the copied file." -ForegroundColor Yellow
                Write-Host ""
            } else {
                Write-Host "[ERROR] Failed to copy file." -ForegroundColor Red
                Read-Host "Press Enter to exit"
                exit 1
            }
        } else {
            Write-Host "Cancelled." -ForegroundColor Yellow
            Read-Host "Press Enter to exit"
            exit 1
        }
    }
}

# Run installation
try {
    Install-Stdlib
} catch {
    Write-Host ""
    Write-Host "[ERROR] Unexpected error: $_" -ForegroundColor Red
    Write-Host $_.ScriptStackTrace -ForegroundColor Red
}

Read-Host "Press Enter to exit"
