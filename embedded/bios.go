package embedded

import (
	_ "embed"
)

//go:embed gba_bios.bin
var BIOSData []byte
