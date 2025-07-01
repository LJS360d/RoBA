//go:build !debug
// +build !debug

package dbg

type noOpDebugLoggerImpl struct{}

// init function for the non-debug build.
// This will be called when the 'debug' tag is NOT active.
func init() {
	// Initialize the global debugLog variable with the no-op implementation.
	debugLog = &noOpDebugLoggerImpl{}
}

// Printf is a no-op when debug logging is disabled.
func (n *noOpDebugLoggerImpl) Printf(format string, a ...interface{}) {
	// Do nothing
}

// Println is a no-op when debug logging is disabled.
func (n *noOpDebugLoggerImpl) Println(a ...interface{}) {
	// Do nothing
}
