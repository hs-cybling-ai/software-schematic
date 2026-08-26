@echo off
setlocal
"%~dp0.ss\bin\ss.exe" serve --project "%~dp0" %*
