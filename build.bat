@echo off
setlocal

echo.
echo  SoundCloud Desktop Builder
echo  ==========================
echo.
echo  1) Build
echo  2) Clean Build
echo  3) Cancel
echo.
set /p choice="  Select option: "

if "%choice%"=="1" goto build
if "%choice%"=="2" goto clean
if "%choice%"=="3" goto :eof
echo  Invalid option.
goto :eof

:clean
echo.
echo  Cleaning...
cargo clean --manifest-path src-tauri\Cargo.toml
echo.

:build
echo.
echo  Building release...
cargo tauri build
if errorlevel 1 (
    echo.
    echo  Build failed.
    pause
    goto :eof
)

set "OUT=src-tauri\target\release\bundle\nsis"
if exist "%OUT%\SoundCloud_1.0.0_x64-setup.exe" (
    copy /y "%OUT%\SoundCloud_1.0.0_x64-setup.exe" "%OUT%\SoundcloudSetup.exe" >nul
)

echo.
echo  Done:
echo    EXE:       src-tauri\target\release\soundcloud.exe
echo    Installer: %OUT%\SoundcloudSetup.exe
echo.
pause
