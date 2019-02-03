	.file	"minimal.c"
	.option nopic
	.text
	.align	1
	.globl	main
	.type	main, @function
main:
	addi	sp,sp,-32
	sd	s0,24(sp)
	addi	s0,sp,32
	mv	a5,a0
	sd	a1,-32(s0)
	sw	a5,-20(s0)
	lw	a5,-20(s0)
	sext.w	a4,a5
	li	a5,1
	bne	a4,a5,.L2
	li	a5,1
	j	.L3
.L2:
	ld	a5,-32(s0)
	addi	a5,a5,8
	ld	a5,0(a5)
	lbu	a5,0(a5)
	mv	a4,a5
	li	a5,97
	bne	a4,a5,.L4
	li	a5,2
	j	.L3
.L4:
	li	a5,0
.L3:
	mv	a0,a5
	ld	s0,24(sp)
	addi	sp,sp,32
	jr	ra
	.size	main, .-main
	.ident	"GCC: (GNU) 8.2.0"
