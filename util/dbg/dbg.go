package dbg

// DebugLogger is an interface that defines our debug logging functions.
// This allows us to have different implementations based on build tags.
type DebugLogger interface {
	Printf(format string, a ...interface{})
	Println(a ...interface{})
}

// Global variable for our debug logger instance.
// This will be initialized by either debug-log.go or nodebug-log.go depending on build tags.
var debugLog DebugLogger

func Printf(format string, a ...interface{}) {
	debugLog.Printf(format, a...)
}

func Println(a ...interface{}) {
	debugLog.Println(a...)
}
