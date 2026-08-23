# GlazeWM Window Info

A PowerShell utility that queries GlazeWM's focused window (`glazewm query focused`) and shows its `Process`, `Class`, and `Title` in a small window with a read-only text box.

## Installation

Run from the project directory:

```powershell
powershell.exe -NoProfile -ExecutionPolicy Bypass -File .\install.ps1
```

This installs the utility to `%LOCALAPPDATA%\GlazeWMWindowInfo`. Run it again to update the installed copy.

## GlazeWM configuration

Edit your existing GlazeWM configuration, normally `%USERPROFILE%\.glzr\glazewm\config.yaml`.

Add this match entry to its existing `window_rules` ignore rule:

```yaml
window_rules:
  - commands: ['ignore']
    match:
      - window_process: { equals: 'powershell' }
        window_title: { equals: 'GlazeWM Window Info' }
```

Add the hotkey to your existing `keybindings` list:

```yaml
keybindings:
  - commands:
      - 'shell-exec --hide-window powershell.exe -NoProfile -STA -ExecutionPolicy Bypass -File "%LOCALAPPDATA%\GlazeWMWindowInfo\glazewm-window-info.ps1"'
    bindings: ['ctrl+alt+i']
```

Reload with `Alt+Shift+R` (the default binding), or run:

```powershell
glazewm command wm-reload-config
```

## Custom GlazeWM path

The script uses `glazewm.exe` from `PATH` by default. To override it, edit this value near the top of the script:

```powershell
$customGlazeWmPath = 'D:\Apps\GlazeWM\glazewm.exe'
```

## Usage

1. Focus the target window.
2. Press `Ctrl+Alt+I`.
3. Select text and press `Ctrl+C`, or click `Copy All`.
4. Press `Esc` or click `Close`.
