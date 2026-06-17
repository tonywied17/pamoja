@echo off
REM Double-click to run the pamoja showcase locally with live reload.
REM Needs Node.js installed. Opens http://localhost:8099 in your browser.
cd /d "%~dp0"
node serve.mjs %*
pause
