package cpu

import "fmt"

func (c *CPU) execute_Thumb(instruction uint16) {
	decoded := DecodeInstruction_Thumb(instruction)
	switch inst := decoded.(type) {
	// Handle DataProcessingInstruction
	case THUMBDataProcessingInstruction:
	default:
		fmt.Printf("Unknown Thumb instruction: 0x%v\n", inst)
	}

}

// #############################
// Thumb Instruction Implementations
// #############################

// Executes Thumb ADD instruction.
func (c *CPU) execAdd_Thumb(instruction uint32) {
	rn := (instruction >> 3) & 0x07 // Bits 3-5 for Rn
	rm := instruction & 0x07        // Bits 0-2 for Rm

	result := c.Registers[rm] + c.Registers[rn]
	c.Registers[rn] = result // Store result in Rn
	fmt.Printf("Thumb ADD R%d, R%d: Result = %d\n", rn, rm, result)
}
