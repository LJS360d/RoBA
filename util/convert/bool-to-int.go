package convert

// BoolToInt converts a boolean value to an integer.
// It returns 0 for false and 1 for true.
func BoolToInt(b bool) int {
	if b {
		return 1
	}
	return 0
}
