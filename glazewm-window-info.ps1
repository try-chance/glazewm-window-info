#Requires -Version 5.1

# GlazeWM Window Info
# Query GlazeWM's focused window and show the result in a window with selectable text.

$ErrorActionPreference = 'Stop'

$appName = 'GlazeWM Window Info'
$mutexName = 'Local\GlazeWMWindowInfo'

# Optional absolute path. Leave empty to search PATH.
$customGlazeWmPath = ''

function Get-GlazeWmPath {
    if (-not [string]::IsNullOrWhiteSpace($customGlazeWmPath)) {
        if (Test-Path -LiteralPath $customGlazeWmPath -PathType Leaf) {
            return $customGlazeWmPath
        }

        throw "Custom GlazeWM CLI path does not exist: $customGlazeWmPath"
    }

    $command = Get-Command 'glazewm.exe' -CommandType Application -ErrorAction SilentlyContinue |
        Select-Object -First 1

    if ($null -ne $command) {
        return $command.Path
    }

    throw 'GlazeWM CLI was not found in PATH. Set $customGlazeWmPath in the script.'
}

# Show an em dash for missing fields.
function Get-DisplayValue($Value) {
    $text = [string] $Value
    if ([string]::IsNullOrWhiteSpace($text)) {
        # U+2014 EM DASH (—).
        return [char] 0x2014
    }

    return $text
}

function Invoke-GlazeWmQuery {
    $startInfo = [System.Diagnostics.ProcessStartInfo]::new()
    $startInfo.FileName = Get-GlazeWmPath
    $startInfo.Arguments = 'query focused'
    $startInfo.UseShellExecute = $false
    $startInfo.CreateNoWindow = $true
    $startInfo.RedirectStandardOutput = $true
    $startInfo.RedirectStandardError = $true
    # GlazeWM writes UTF-8 JSON; do not inherit the hidden console's code page.
    $startInfo.StandardOutputEncoding = [System.Text.Encoding]::UTF8
    $startInfo.StandardErrorEncoding = [System.Text.Encoding]::UTF8

    $process = [System.Diagnostics.Process]::new()
    $process.StartInfo = $startInfo

    try {
        if (-not $process.Start()) {
            throw 'Windows could not start the GlazeWM CLI.'
        }

        # Drain both streams asynchronously to avoid pipe deadlocks.
        $stdoutTask = $process.StandardOutput.ReadToEndAsync()
        $stderrTask = $process.StandardError.ReadToEndAsync()

        if (-not $process.WaitForExit(10000)) {
            $process.Kill()
            $process.WaitForExit()
            throw 'GlazeWM did not respond within 10 seconds.'
        }

        $stdout = $stdoutTask.GetAwaiter().GetResult()
        $stderr = $stderrTask.GetAwaiter().GetResult()
        $exitCode = $process.ExitCode
    }
    finally {
        $process.Dispose()
    }

    if ($exitCode -ne 0) {
        # Collapse whitespace into a single line.
        $message = ($stderr -replace '\s+', ' ').Trim()
        if ([string]::IsNullOrWhiteSpace($message)) {
            $message = "exit code $exitCode"
        }
        throw "GlazeWM query failed: $message"
    }

    try {
        $response = $stdout | ConvertFrom-Json
    }
    catch {
        throw 'GlazeWM returned an invalid JSON response.'
    }

    if ($null -eq $response) {
        throw 'GlazeWM returned an empty JSON response.'
    }

    if ($response.success -eq $false) {
        $message = [string] $response.error
        if ([string]::IsNullOrWhiteSpace($message)) {
            $message = 'GlazeWM returned an unknown error.'
        }
        throw $message
    }

    $focused = $response.data.focused
    if ($null -eq $focused) {
        throw 'GlazeWM did not return a focused window.'
    }

    return [PSCustomObject]@{
        Process = Get-DisplayValue $focused.processName
        Class   = Get-DisplayValue $focused.className
        Title   = Get-DisplayValue $focused.title
    }
}

function Show-WindowInfo {
    param(
        [Parameter(Mandatory)]
        [string] $Content,

        [switch] $IsError
    )

    Add-Type -AssemblyName System.Drawing
    Add-Type -AssemblyName System.Windows.Forms

    if ($null -eq ('GlazeWmWindowInfo.NativeMethods' -as [type])) {
        Add-Type @'
using System;
using System.Runtime.InteropServices;

namespace GlazeWmWindowInfo
{
    public static class NativeMethods
    {
        [DllImport("user32.dll")]
        public static extern bool ShowWindow(IntPtr windowHandle, int command);
    }
}
'@
    }

    [System.Windows.Forms.Application]::EnableVisualStyles()

    # Resizable dialog without a maximize button.
    $form = [System.Windows.Forms.Form]::new()
    $form.Text = $appName
    $form.StartPosition = 'CenterScreen'
    $form.ClientSize = [System.Drawing.Size]::new(560, 300)
    $form.MinimumSize = [System.Drawing.Size]::new(460, 260)
    $form.MaximizeBox = $false
    $form.ShowIcon = $false
    $form.Font = [System.Drawing.Font]::new('Segoe UI', 10)

    $heading = [System.Windows.Forms.Label]::new()
    $heading.AutoSize = $true
    $heading.Location = [System.Drawing.Point]::new(16, 16)
    $heading.Font = [System.Drawing.Font]::new(
        $form.Font.FontFamily,
        12,
        [System.Drawing.FontStyle]::Bold
    )
    $heading.Text = if ($IsError) { 'Query failed' } else { 'Focused window' }

    # Read-only text remains selectable and copyable.
    $details = [System.Windows.Forms.RichTextBox]::new()
    $details.Location = [System.Drawing.Point]::new(16, 50)
    $details.Size = [System.Drawing.Size]::new(528, 195)
    $details.Anchor = 'Top, Bottom, Left, Right'
    $details.ReadOnly = $true
    $details.DetectUrls = $false
    $details.WordWrap = $true
    $details.BackColor = [System.Drawing.SystemColors]::Window
    $details.Text = $Content

    $copyButton = [System.Windows.Forms.Button]::new()
    $copyButton.Location = [System.Drawing.Point]::new(360, 260)
    $copyButton.Size = [System.Drawing.Size]::new(88, 28)
    $copyButton.Anchor = 'Bottom, Right'
    $copyButton.Text = 'Copy All'
    $copyButton.Add_Click({
            [System.Windows.Forms.Clipboard]::SetText($details.Text)
            $copyButton.Text = 'Copied'
        })

    $closeButton = [System.Windows.Forms.Button]::new()
    $closeButton.Location = [System.Drawing.Point]::new(456, 260)
    $closeButton.Size = [System.Drawing.Size]::new(88, 28)
    $closeButton.Anchor = 'Bottom, Right'
    $closeButton.Text = 'Close'
    $closeButton.Add_Click({ $form.Close() })

    $form.Controls.AddRange(@($heading, $details, $copyButton, $closeButton))
    $form.CancelButton = $closeButton # Esc closes the dialog.

    $form.Add_Shown({
            # GlazeWM hides PowerShell at startup; explicitly show only this dialog.
            [void] [GlazeWmWindowInfo.NativeMethods]::ShowWindow($form.Handle, 5)
            $form.TopMost = $true
            $form.Activate()
            $form.TopMost = $false
            $details.Focus()
        })

    try {
        [void] $form.ShowDialog()
    }
    finally {
        $form.Dispose()
    }
}

# Exit if another instance still owns the named mutex.
$isFirstInstance = $false
$mutex = [System.Threading.Mutex]::new($true, $mutexName, [ref] $isFirstInstance)

if (-not $isFirstInstance) {
    $mutex.Dispose()
    exit 0
}

try {
    try {
        # Query before creating the dialog so it never captures itself.
        $windowInfo = Invoke-GlazeWmQuery
        $content = @(
            "Process: $($windowInfo.Process)"
            "Class:   $($windowInfo.Class)"
            "Title:   $($windowInfo.Title)"
        ) -join [Environment]::NewLine

        Show-WindowInfo -Content $content
    }
    catch {
        # Show a short error without a PowerShell stack trace.
        $message = ($_.Exception.Message -replace '\s+', ' ').Trim()
        if ($message.Length -gt 400) {
            # U+2026 HORIZONTAL ELLIPSIS (…).
            $message = $message.Substring(0, 399) + [char] 0x2026
        }

        Show-WindowInfo -Content "Failed to query focused window:`r`n`r`n$message" -IsError
    }
}
finally {
    $mutex.ReleaseMutex()
    $mutex.Dispose()
}
