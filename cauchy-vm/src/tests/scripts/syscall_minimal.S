	.file	"syscall_minimal.c"
	.option nopic
	.text
	.align	1
.MSG:
	.string	"DEADBEEF is happyBEEF"
	.align	3
.TXID:
	.string	"RECVR"
	.text
	.align	1
	.globl	_start
	.type	_start, @function
_start:
	lui a5,%hi(.TXID)
	addi	a3,a5,%lo(.TXID)
	li a4, 5
	lui	a5,%hi(.MSG)
	addi	a5,a5,%lo(.MSG)
	li a6, 21
	li a7, 0xCBFF
	ecall
	li a0, 0
	li a7, 93
	ecall
	.size	_start, .-_start