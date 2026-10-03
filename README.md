# BeniUnityEmu

**BeniUnityEmu** – Experimental Unity emulation runtime for old Android & iOS Unity games on Windows.

**Beni App Picker** – Modern GUI launcher (`BeniAppPicker.exe`).

Made by **Beni**.

## Binaries

| Binary | Description |
|--------|-------------|
| `BeniAppPicker.exe` | Graphical launcher (egui) |
| `BeniUnityEmu.exe` | CLI runtime |

## Build

```bash
cargo build --release
# CLI only:
cargo build --release --no-default-features --bin BeniUnityEmu
```

Outputs: `target/release/BeniAppPicker.exe`, `target/release/BeniUnityEmu.exe`

## Folders

```
BeniApps/   ← APK + IPA
BeniFonts/
BeniSO/
BeniDyLIB/
cache/
```

## License

MIT
