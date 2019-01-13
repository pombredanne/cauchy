	.file	"syscalls.c"
	.option nopic
	.text
	.section	.rodata
	.align	3
.LC0:
	.string	"ABCDEFGH"
	.text
	.align	1
	.globl	main
	.type	main, @function
main:
	addi	sp,sp,-32
	sd	s0,24(sp)
	addi	s0,sp,32
	lui	a5,%hi(.LC0)
	ld	a5,%lo(.LC0)(a5)
	sd	a5,-24(s0)
	addi	a4,s0,-24
	li	a3,8
 #APP
# 6 "syscalls.c" 1
	mv a5, a3
	mv a6, a4
	li a7, 0xCBFF
	ecall
	
# 0 "" 2
 #NO_APP
	li	a5,0
	mv	a0,a5
	ld	s0,24(sp)
	addi	sp,sp,32
	jr	ra
	.size	main, .-main
	.ident	"GCC: (GNU) 8.2.0"
