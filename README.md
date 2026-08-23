# GlazeWM Window Info

一个用于查看 `glazewm query focused` 完整结果的 Windows 托盘工具。

## 功能

- 常驻系统托盘，主窗口可隐藏到托盘。
- 在目标窗口获得焦点时按 `Ctrl+Alt+I`。
- 调用 `glazewm query focused`，使用 GlazeWM 自己识别的字段。
- 单独显示常用的 `processName`、`className` 和 `title` 字段。
- 格式化显示 GlazeWM 返回的完整 JSON，并高亮常用字段；支持按钮复制或用鼠标选择后复制。
- 关闭主窗口时隐藏到托盘；通过托盘菜单或界面按钮退出。
- 单实例运行。
- Release 版本不显示控制台窗口。

## 安装和运行

开发模式：

```powershell
cargo run
```

推荐安装到 Cargo 的全局命令目录：

```powershell
cargo install --path . --locked
```

`cargo install` 默认使用 Release 配置。安装完成后，程序通常位于：

```text
C:\Users\31640\.cargo\bin\glazewm-window-info.exe
```

确认命令可以通过 `PATH` 找到：

```powershell
Get-Command glazewm-window-info
```

修改代码后，需要重新执行 `cargo install --path . --locked` 才能更新已安装的程序。

如果只想在项目内构建 Release：

```powershell
cargo build --release
```

生成的程序位于：

```text
target\release\glazewm-window-info.exe
```

## GlazeWM 配置

把下面的条目合并进现有的 `general.startup_commands`。不要创建重复的
`general:` 节点。

```yaml
general:
  startup_commands:
    - 'shell-exec glazewm-window-info'
```

如果 GlazeWM 找不到命令，请确认 `C:\Users\31640\.cargo\bin` 已加入 `PATH`，然后重启 GlazeWM。

把工具自身加入现有的 `window_rules:` 列表：

```yaml
window_rules:
  - commands: ['ignore']
    match:
      - window_process: { equals: 'glazewm-window-info' }
```

重新加载配置：

```powershell
glazewm command wm-reload-config
```

## 使用

1. 启动工具。
2. 激活要检查的窗口。
3. 按 `Ctrl+Alt+I`。
4. 在弹出的窗口中查看常用字段和完整 JSON。
5. 用鼠标选择需要的文本，按 `Ctrl+C` 复制。

编写规则时通常优先参考 Process 和 Class。Title 更精确，但很多应用的标题会随当前文档或页面改变。

如果目标窗口已经被 GlazeWM `ignore`，`query focused` 通常无法再取得它；请先暂时移除原规则并重新加载配置。
