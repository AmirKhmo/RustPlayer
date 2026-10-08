@echo off
title RustPlayer build
powershell -NoProfile -ExecutionPolicy Bypass -File "%~dp0build-windows.ps1" %*
echo.
pause
