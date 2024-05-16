# macro-maker
Define global keyboard shortcuts to execute shell commands

## Platforms
- ### ✔️ Linux
- ### ✔️ Windows
- ### ❌ MacOS

Binary expects that file `dispatch.toml` exists in the same directory as the executable, `dispatch.log` will be automatically generated on run.

## Example `dispatch.toml`
##### !!! Alt+Shift+Control+KeyE is the BUILT-IN macro to quit the program !!!
```
# firefox
[[commands]]
modifiers = { alt = true, shift = true, control = true }
hotkey = "KeyS"
script = "firefox example.com"

# test box
[[commands]]
modifiers = { alt = true, shift = true, control = true }
hotkey = "KeyT"
script = "PowerShell -Command \"Add-Type -AssemblyName PresentationFramework;[System.Windows.MessageBox]::Show('Daemon Running')\""
```

## Modifier options
- 'alt' is equivalent to the option key on some platforms,
- whereas 'meta' is equivalent to the command key/windows key/super key
```
modifiers = { alt = true, meta = true, shift = true, control = true }
```
