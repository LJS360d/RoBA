package main

import (
	"GoBA/internal/bus"
	"GoBA/internal/cartridge"
	"GoBA/internal/cpu"
	"GoBA/internal/io"
	"GoBA/internal/memory"
	"GoBA/internal/ppu"
	"GoBA/util/dbg"
	"flag"
	"image"
	"image/png"
	"log"
	"os"
	"runtime"
	"time"
)

func main() {
	fp := flag.String("rom", "", "Path to ROM file")
	flag.Parse()
	if *fp == "" {
		log.Fatal("ROM file path is required")
	}

	romData, err := os.ReadFile(*fp)
	if err != nil {
		log.Fatal(err)
	}

	// Initialize components
	bios := memory.NewBIOS()
	ewram := memory.NewEWRAM()
	iwram := memory.NewIWRAM()
	ppu := ppu.NewPPU()
	cart := cartridge.NewCartridge(romData)
	regs := io.NewIORegs()
	// Create bus
	bus := bus.NewBus(bios, ewram, iwram, ppu, cart, regs)
	// Connect components to bus
	ppu.SetBus(bus)

	// Create CPU
	cpu := cpu.NewCPU(bus)
	cpu.Reset()

	// Main emulation loop
	frameCount := 0
	lastTime := time.Now()

	for {
		// Run CPU for one instruction
		cpu.Step()

		// Tick other components
		bus.Tick(1)

		// Check if frame is ready
		if ppu.IsFrameReady() {
			frameCount++
			ppu.ResetFrameReady()

			// Save first frame to file
			if frameCount == 1 {
				saveFrame(ppu.Frame, "first_frame.png")
			}
		}

		// Simple FPS limiting
		if time.Since(lastTime) >= time.Second {
			dbg.Printf("FPS: %d\n", frameCount)
			frameCount = 0
			lastTime = time.Now()
		}

		// Yield to other goroutines
		runtime.Gosched()
	}
}

func saveFrame(img *image.RGBA, filename string) {
	file, err := os.Create(filename)
	if err != nil {
		log.Fatal(err)
	}
	defer file.Close()

	if err := png.Encode(file, img); err != nil {
		log.Fatal(err)
	}
	log.Printf("Saved first frame to %s", filename)
}
