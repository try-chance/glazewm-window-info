# GlazeWM Window Info

A single-file PowerShell utility that queries GlazeWM's focused window and shows its `Process`, `Class`, and `Title` in a copyable dialog.

- No build step or PowerShell modules required.
- Exits when the dialog closes.
- Stops a query after 10 seconds.
- Uses a mutex to prevent concurrent queries and duplicate dialogs.

## GlazeWM configuration

Add this match entry to your existing `window_rules` ignore rule:

```yaml
window_rules:
  - commands: ['ignore']
    match:
      - window_process: { equals: 'powershell' }
        window_title: { equals: 'GlazeWM Window Info' }
```

Add the hotkey to your existing `keybindings` list. Update the script path as needed:

```yaml
keybindings:
  - commands:
      - 'shell-exec --hide-window powershell.exe -NoProfile -STA -ExecutionPolicy Bypass -File "C:\Users\31640\Desktop\glazewm-window-inspector\glazewm-window-info.ps1"'
    bindings: ['ctrl+alt+i']
```

Merge these entries into the existing `window_rules:` and `keybindings:` sections. Do not add duplicate top-level sections.

`--hide-window` hides the console. Do not add `powershell.exe -WindowStyle Hidden`, because it may also hide the dialog.

Reload the configuration:

```powershell
glazewm command wm-reload-config
```

## GlazeWM CLI path

The script searches `PATH` for `glazewm.exe` by default. To use a custom location, edit this value near the top of the script:

```powershell
$customGlazeWmPath = 'D:\Apps\GlazeWM\glazewm.exe'
```

A custom path takes precedence over `PATH`. Leave it empty to use `PATH` only. An invalid custom path is shown as an error in the dialog.

## Usage

1. Focus the target window.
2. Press `Ctrl+Alt+I`.
3. Select text and press `Ctrl+C`, or click `Copy All`.
4. Press `Esc` or click `Close`.

To run the script manually:

```powershell
powershell.exe -NoProfile -STA -ExecutionPolicy Bypass -File .\glazewm-window-info.ps1
```
