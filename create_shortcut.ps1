$WshShell = New-Object -ComObject WScript.Shell
$DesktopPath = [Environment]::GetFolderPath("Desktop")
$ShortcutPath = Join-Path $DesktopPath "Forge Editor.lnk"
$Shortcut = $WshShell.CreateShortcut($ShortcutPath)
$Shortcut.TargetPath = "C:\Users\osaretin\Downloads\forge\target\release\forge.exe"
$Shortcut.WorkingDirectory = "C:\Users\osaretin\Downloads\forge"
$Shortcut.IconLocation = "C:\Users\osaretin\Downloads\forge\forge.ico,0"
$Shortcut.Description = "Forge - GPU-Accelerated Code Editor"
$Shortcut.Save()
Write-Output "Desktop shortcut created at: $ShortcutPath"
