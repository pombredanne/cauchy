	.text
	.globl	_start
	.type	_start, @function
_start:
	li a5, 32           # TXID size
    li a6, 32+64        # MSG Size
    li a7, 0xCBFA		# Syscall __vm_send()
	ecall				# Exec syscall
	li a7, 0xCBF8		# Load block before death
	ecall				# Exec syscall
	li a0, 0			# Load retval
	li a7, 93			# Load exit syscall
	ecall				# Excec syscall