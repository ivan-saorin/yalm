@echo off
REM Set up MSVC + Windows SDK environment for Rust builds.
REM Requires: VS 2022 Community with "Desktop development with C++" workload.
call "C:\Program Files\Microsoft Visual Studio\2022\Community\VC\Auxiliary\Build\vcvars64.bat" >nul 2>&1
cd /d D:\workspace\projects\yalm
cargo build --bin yalm-eval %*
