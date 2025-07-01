//go:build debug
// +build debug

package dbg

import (
	"fmt"
	"log"
	"os"
)

type debugLoggerImpl struct {
	logger *log.Logger
}

// init function for the debug build.
// This will be called when the 'debug' tag is active.
func init() {
	// Initialize the global debugLog variable with the actual logging implementation.
	// We use log.New to create a logger that writes to stderr (or any io.Writer)
	// and includes file/line number for easy debugging.
	debugLog = &debugLoggerImpl{
		logger: log.New(os.Stderr, "", log.Lshortfile),
	}
}

// Printf implements the Printf method of the DebugLogger interface.
func (d *debugLoggerImpl) Printf(format string, a ...interface{}) {
	d.logger.Output(3, fmt.Sprintf(format, a...)) // calldepth 2 to get caller's file/line
}

// Println implements the Println method of the DebugLogger interface.
func (d *debugLoggerImpl) Println(a ...interface{}) {
	d.logger.Output(3, fmt.Sprintln(a...)) // calldepth 2 to get caller's file/line
}
