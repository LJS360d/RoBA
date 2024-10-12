# GoBA

GoBA is a GBA emulator written in Go. It is designed to be a cross-platform, cross-architecture emulator that can run on desktops, mobile devices, and web browsers.

```md
/GoBA
├── /cmd             # For different entry points (CLI, WASM, etc.)
│   ├── /desktop     # Desktop-specific entry point
│   └── /web         # WASM-specific entry point for web-based emulator
├── /cpu             # ARM7TDMI CPU emulation
├── /gpu             # Graphics, including the GBA's PPU (rendering)
├── /apu             # Audio processing unit (sound generation)
├── /input           # Input handling (keyboard, gamepad, etc.)
├── /rom             # ROM file loading and cartridge handling
├── /memory          # Memory management (RAM, VRAM, IO, etc.)
├── /platform        # Platform abstraction (WebAssembly, Desktop)
│   ├── /desktop     # SDL, OpenGL bindings, etc.
│   └── /wasm        # WebAssembly bindings (canvas, audio, input)
├── /ui              # User Interface, handling game windows or browser canvas
└── /util            # Utility functions and helpers
```

## List of Milestones:
- ~~Core infrastructure setup (cross-platform setup, project structure)~~
- ROM loading and memory management
- CPU emulation (ARM7TDMI)
- Input handling
- Graphics rendering (GPU/PPU)
- Sound emulation (APU)
- Save states and cartridge emulation
- Debugging tools (stepping, breakpoints)
- BIOS emulation (optional)

## Gba ROM memory Map

- BIOS (0x00000000 - 0x00003FFF): 16KB, the bootstrap ROM.
- EWRAM (0x02000000 - 0x0203FFFF): 256KB, external work RAM.
- IWRAM (0x03000000 - 0x03007FFF): 32KB, internal work RAM.
- IO Registers (0x04000000 - 0x040003FE): I/O registers for video, sound, - etc.
- VRAM (0x06000000 - 0x06017FFF): 96KB, video RAM.
- OAM (0x07000000 - 0x070003FF): 1KB, object attribute memory for sprites.
- ROM (0x08000000 - 0x09FFFFFF): Up to 32MB, cartridge ROM.