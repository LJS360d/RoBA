package main

import (
	"fmt"
	"os"

	"GoBA/cpu"
	"GoBA/memory"
	"GoBA/rom"
)

func main() {
	if len(os.Args) < 2 {
		fmt.Println("Usage: GoBA <path-to-rom.gba>")
		os.Exit(1)
	}

	romPath := os.Args[1]
	gbaRom, err := rom.Load(romPath)
	if err != nil {
		fmt.Printf("Error loading ROM: %v\n", err)
		os.Exit(1)
	}

	// Initialize memory with the loaded ROM
	mem := memory.NewMemory(gbaRom.Data)
	// Initialize CPU with the memory
	cpu := cpu.NewCPU(mem)
	// Start the CPU execution loop
	cpu.Run()
}
