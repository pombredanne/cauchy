	.file	"syscalls2.c"
	.option nopic
	.text
	.align	1
	.globl	atoi
	.type	atoi, @function
atoi:
	addi	sp,sp,-48
	sd	s0,40(sp)
	addi	s0,sp,48
	sd	a0,-40(s0)
	sw	zero,-20(s0)
	sw	zero,-24(s0)
	j	.L2
.L3:
	lw	a4,-20(s0)
	mv	a5,a4
	slliw	a5,a5,2
	addw	a5,a5,a4
	slliw	a5,a5,1
	sext.w	a4,a5
	lw	a5,-24(s0)
	ld	a3,-40(s0)
	add	a5,a3,a5
	lbu	a5,0(a5)
	sext.w	a5,a5
	addw	a5,a4,a5
	sext.w	a5,a5
	addiw	a5,a5,-48
	sw	a5,-20(s0)
	lw	a5,-24(s0)
	addiw	a5,a5,1
	sw	a5,-24(s0)
.L2:
	lw	a5,-24(s0)
	ld	a4,-40(s0)
	add	a5,a4,a5
	lbu	a5,0(a5)
	bnez	a5,.L3
	lw	a5,-20(s0)
	mv	a0,a5
	ld	s0,40(sp)
	addi	sp,sp,48
	jr	ra
	.size	atoi, .-atoi
	.section	.rodata
	.align	3
.LC0:
	.string	"hello"
	.zero	2
	.text
	.align	1
	.globl	main
	.type	main, @function
main:
	addi	sp,sp,-80
	sd	s0,72(sp)
	addi	s0,sp,80
	mv	a5,a0
	sd	a1,-80(s0)
	sw	a5,-68(s0)
	lui	a5,%hi(.LC0)
	lw	a4,%lo(.LC0)(a5)
	sw	a4,-24(s0)
	addi	a5,a5,%lo(.LC0)
	lhu	a5,4(a5)
	sh	a5,-20(s0)
	sh	zero,-18(s0)
	addi	a2,s0,-56
	li	a1,32
	addi	a0,s0,-24
	li	t1,5
 #APP
# 23 "syscalls2.c" 1
	mv a3, a2
	mv a4, a1
	mv a5, a0
	mv a6, t1
	li a7, 0xCBFE
	ecall
	
# 0 "" 2
 #NO_APP
	addi	a4,s0,-56
	li	a3,32
 #APP
# 24 "syscalls2.c" 1
	mv a5, a3
	mv a6, a4
	li a7, 0xCBFF
	ecall
	
# 0 "" 2
# 25 "syscalls2.c" 1
	li a0, 0
	li a7, 93
	ecall
	
# 0 "" 2
 #NO_APP
	li	a5,0
	mv	a0,a5
	ld	s0,72(sp)
	addi	sp,sp,80
	jr	ra
	.size	main, .-main
	.ident	"GCC: (GNU) 8.2.0"
