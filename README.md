# BeniUnityEmu

**BeniUnityEmu** – Experimental Unity emulation / compatibility runtime for old Android & iOS Unity games on Windows.

Made by **Beni**.

## Status (v0.1.0 – Stage 2)

Solid foundations. **No game execution yet.**

### Working
- APK / IPA detection and ZIP-based loading
- Structure listing (manifest, dex, assets, lib/, architectures)
- Detection of `libunity.so`, `libmain.so`, `libmono.so`
- Cache system with `metadata.json`
- Basic ELF header parsing (class, arch, endian, entry)
- Basic Mach-O magic detection
- **Virtual Memory Manager** (map/unmap, read/write 8/16/32/64)
- ARM32 interpreter with MOV, ADD, SUB, CMP, AND, ORR, EOR, LSL/LSR/ASR, LDR, STR, PUSH, POP, B, BL, BX, BLX + condition codes
- ARM64 stub
- Android / iOS / Unity runtime state machines
- JNI internal stubs (FindClass, GetMethodID, etc.)
- GameSession orchestration
- Interactive console launcher + CLI flags
- GitHub Actions → ZIP artifacts

### Stubs / Not implemented
- Full ARM instruction set / real libunity execution
- Real Unity engine initialization
- Full Android JNI / Activity lifecycle
- OpenGL ES translation
- Audio / full Input backends
- iOS dyld / full Mach-O loading
- Actual game execution

## Build

```bash
cargo build
cargo build --release
```

## Usage

```bash
./BeniUnityEmu
./BeniUnityEmu --scan
./BeniUnityEmu --inspect BadPiggies.apk
./BeniUnityEmu --extract BadPiggies.apk
./BeniUnityEmu --run BadPiggies.apk
```

Place APKs in `BeniAPK_Apps/` and IPAs in `BeniIOS_Apps/`.

## License

MIT
