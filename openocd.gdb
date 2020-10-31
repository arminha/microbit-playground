# Connects GDB to OpenOCD server port
target remote :3333
# Enable semihosting
monitor arm semihosting enable
# (optional) Unmangle function names when debugging
set print asm-demangle on
set print pretty on
# Load your program, breaks at entry
load
# (optional) Add breakpoint at function
break main
# Continue with execution
continue
