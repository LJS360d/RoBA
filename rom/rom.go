package rom

import (
	"fmt"
	"os"
)

type ROM struct {
	Data []byte
}

// Load loads a GBA ROM file into memory.
func Load(path string) (*ROM, error) {
	data, err := os.ReadFile(path)
	if err != nil {
		return nil, fmt.Errorf("unable to read ROM file: %v", err)
	}

	if len(data) == 0 {
		return nil, fmt.Errorf("ROM file is empty")
	}

	return &ROM{Data: data}, nil
}
