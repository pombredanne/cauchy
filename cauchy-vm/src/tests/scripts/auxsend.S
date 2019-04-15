	.text
	.globl	_start
	.type	_start, @function
_start:
	li a5, 32           # TXID size
    li a6, 32+64        # MSG Size
    li a7, 0xCBFA		# Syscall __vm_send()
	ecall				# Exec syscall