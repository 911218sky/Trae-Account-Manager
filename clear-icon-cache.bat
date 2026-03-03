@echo off
echo Clearing Windows icon cache...

:: Stop Windows Explorer
taskkill /f /im explorer.exe

:: Delete icon cache files
cd /d %userprofile%\AppData\Local\Microsoft\Windows\Explorer
attrib -h IconCache.db
del IconCache.db
del iconcache_*.db

:: Restart Windows Explorer
start explorer.exe

echo Icon cache cleared! Please restart your computer for full effect.
pause
