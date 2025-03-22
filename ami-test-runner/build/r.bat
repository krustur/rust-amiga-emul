@echo off

pushd ..

ECHO ===[ Running test-runner.exe! ]===
start "test-runner.exe" %WINUAEPATH% ^
 -f "%BUILDPATH%\build\runner_config.uae" ^
 -serlog ^
 -s use_gui=no ^
 -s kickstart_rom_file=%KICKSTART_ROM_FILE% ^
 -s filesystem2="rw,dh0:System:%BUILDPATH%\winuae,0" ^
 -s uaehf0="dir,rw,dh0:System:%BUILDPATH%\winuae,0"

popd
