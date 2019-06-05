	.text
	.globl	_start
	.type	_start, @function
_start:
	li a7, 0xCBF8		# Load block before death
	ecall				# Exec syscall
	li a0, 0			# Load retval
	li a7, 93			# Load exit syscall
	ecall				# Excec syscall