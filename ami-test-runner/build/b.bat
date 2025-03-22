@echo off

echo ===[ Build test-runner.exe! ]===
SET link_target=_amiga\test-runner.exe

SET srcdir=..\src
SET objdir=_amiga

SET symbols=
REM -DAMIGA -DDEBUG -DSERIAL_OUTPUT

SET gcc=m68k-amigaos-gcc
SET gcc_opts=-Wall -Os -fomit-frame-pointer -msmall-code -save-temps=obj -m68020 -I../src
SET gcc_link_opts=

SET asm=vasmm68k_mot
SET asm_opts=-esc -no-opt -m68020 -quiet -I%AMIGAGCCPATH%/m68k-amigaos/ndk-include/

if not exist %objdir% mkdir %objdir%

ECHO ===[ Compiling! ]===
%gcc% %gcc_opts% %symbols% -c -o %objdir%\main.obj %srcdir%\main.c

ECHO ===[ Assembling! ]===
%asm% %asm_opts% %symbols% -Fhunk -o %objdir%\test_runner-s.obj %srcdir%\test_runner.s

ECHO ===[ Linking! ]===
%gcc% %gcc_link_opts% -g ^
 %objdir%\main.obj ^
 %objdir%\test_runner-s.obj ^
 -Xlinker -Map=%objdir%/amiga.map -o %link_target%

cp %link_target% ..\winuae\