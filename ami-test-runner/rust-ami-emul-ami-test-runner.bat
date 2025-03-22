@echo off

SET BUILDPATH=%1
SET KICKSTART_ROM_FILE=%2
SET AMIGAGCCPATH=%3
SET VBCC=%3
set WINUAEPATH=%4

@set PREFIX=%3
path %prefix%\bin\;%path%

REM "c:\Program Files (x86)\Microsoft Visual Studio\2019\Enterprise\Common7\Tools\VsDevCmd.bat" -arch=amd64 -host_arch=amd64

echo  . . . Remember to launch %prefix%\cmdline.bat at least once! . . .
echo.

@echo on

@cmd /K cd /d %BUILDPATH%/build
