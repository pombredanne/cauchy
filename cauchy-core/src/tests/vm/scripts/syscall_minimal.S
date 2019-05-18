.MSG: 
	.string	"DEADBEEF is happyBEEF"
	.text
.TXID:
	.string	"RECVR"
	.text
	.globl	_start
	.type	_start, @function
_start:
	lui a5,%hi(.TXID)			# Upper addr of TXID
	addi	a3,a5,%lo(.TXID)	# Lower addr of TXID
	li a4, 5					# Size of TXID
	lui	a5,%hi(.MSG)			# Upper addr of MSG
	addi	a5,a5,%lo(.MSG)		# Lower addr of MSG
	li a6, 21					# Length of MSG
	li a7, 0xCBFF				# Syscall __vm_send()
	ecall						# Exec syscall