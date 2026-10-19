# Pangoland target (Stage G)

## Game

- **Title:** Pangoland (Studio Pango)
- **Unity:** 4.6.1f1 (`d1db7a1b5196`) — claimed; verify on your files
- **Package:** `StudioPango.Pangoland_2015.313.1049.2457_neutral_~_3jcsvwjpet1qj.AppxBundle`
- **Arch:** x86
- **Host:** Windows 10 Mobile-origin APPX/XAML
- **Graphics (original):** Direct3D 11
- **Managed:** `Assembly-CSharp.dll`, `TouchScript.dll`
- **Native:** `UnityPlayer.dll`

## Supply the package

This repository does **not** ship the game files.

1. Extract the AppxBundle / install folder on your PC.
2. Either:
   - Copy extracted contents to `BeniApps/PangoLand/`, or
   - Set environment variable:
     ```bat
     set BENI_PANGOLAND_PATH=C:\path\to\extracted\pangoland
     ```
3. Run inspection:
   ```bat
   cargo run --bin BeniUnityEmu -- pangoland-inspect
   ```
   or from tests / library:
   ```rust
   beni_unity_emu::unity::inspect_pangoland_target(None);
   ```

## Expected markers

| File | Role |
|------|------|
| `AppxManifest.xml` | UWP identity / arch |
| `UnityPlayer.dll` | Native Unity player |
| `*/Managed/Assembly-CSharp.dll` | Game scripts (compiled) |
| `*/Managed/TouchScript.dll` | TouchScript |
| `*_Data/` or package `Data/` | Assets / scenes |

## Honesty rules

- Presence of DLLs ≠ game execution.
- Do not claim Pangoland runs inside BeniUnityEmu until managed + native guest paths execute real scripts.
- Missing Behaviour script references must be diagnosed from serialized assets + type names, not guessed.
