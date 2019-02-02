	.file	"freeze.c"
	.option nopic
	.text
	.align	1
	.globl	__vm_retbytes
	.type	__vm_retbytes, @function
__vm_retbytes:
	addi	sp,sp,-32
	sd	s0,24(sp)
	addi	s0,sp,32
	sd	a0,-24(s0)
	mv	a5,a1
	sw	a5,-28(s0)
	ld	a4,-24(s0)
	lw	a3,-28(s0)
 #APP
# 3 "vm.h" 1
	mv a5, a3
	mv a6, a4
	li a7, 0xCBFF
	ecall
	
# 0 "" 2
 #NO_APP
	nop
	ld	s0,24(sp)
	addi	sp,sp,32
	jr	ra
	.size	__vm_retbytes, .-__vm_retbytes
	.align	1
	.globl	__vm_exit
	.type	__vm_exit, @function
__vm_exit:
	addi	sp,sp,-32
	sd	s0,24(sp)
	addi	s0,sp,32
	mv	a5,a0
	sw	a5,-20(s0)
	lw	a5,-20(s0)
 #APP
# 15 "vm.h" 1
	mv a0, a5
	li a7, 93
	ecall
	
# 0 "" 2
 #NO_APP
	nop
	ld	s0,24(sp)
	addi	sp,sp,32
	jr	ra
	.size	__vm_exit, .-__vm_exit
	.align	1
	.globl	__vm_call
	.type	__vm_call, @function
__vm_call:
	addi	sp,sp,-48
	sd	s0,40(sp)
	addi	s0,sp,48
	sd	a0,-24(s0)
	sd	a1,-32(s0)
	mv	a5,a2
	sd	a3,-48(s0)
	sw	a5,-36(s0)
	mv	a5,a4
	sw	a5,-40(s0)
	ld	a1,-48(s0)
	lw	a0,-40(s0)
	ld	t1,-32(s0)
	lw	t3,-36(s0)
	ld	t4,-24(s0)
 #APP
# 26 "vm.h" 1
	mv a2, t4
	mv a3, a1
	mv a4, a0
	mv a5, t1
	mv a6, t3
	li a7, 0xCBFE
	ecall
	
# 0 "" 2
 #NO_APP
	nop
	ld	s0,40(sp)
	addi	sp,sp,48
	jr	ra
	.size	__vm_call, .-__vm_call
	.align	1
	.globl	__vm_getrand
	.type	__vm_getrand, @function
__vm_getrand:
	addi	sp,sp,-32
	sd	s0,24(sp)
	addi	s0,sp,32
	sd	a0,-24(s0)
	mv	a5,a1
	sw	a5,-28(s0)
	ld	a4,-24(s0)
	lw	a3,-28(s0)
 #APP
# 58 "vm.h" 1
	mv a5, a3
	mv a6, a4
	li a7, 0xCBFD
	ecall
	
# 0 "" 2
 #NO_APP
	nop
	ld	s0,24(sp)
	addi	sp,sp,32
	jr	ra
	.size	__vm_getrand, .-__vm_getrand
	.align	1
	.globl	__vm_gettime
	.type	__vm_gettime, @function
__vm_gettime:
	addi	sp,sp,-32
	sd	s0,24(sp)
	addi	s0,sp,32
	sd	a0,-24(s0)
	ld	a4,-24(s0)
 #APP
# 70 "vm.h" 1
	mv a5, a4
	li a7, 0xCBFC
	ecall
	
# 0 "" 2
 #NO_APP
	nop
	ld	s0,24(sp)
	addi	sp,sp,32
	jr	ra
	.size	__vm_gettime, .-__vm_gettime
	.align	1
	.globl	main
	.type	main, @function
main:
	addi	sp,sp,-48
	sd	s0,40(sp)
	addi	s0,sp,48
	mv	a5,a0
	sd	a1,-48(s0)
	sw	a5,-36(s0)
	sd	zero,-24(s0)
	addi	a4,s0,-24
	li	a3,8
 #APP
# 8 "freeze.c" 1
	mv a5, a4
	mv a6, a3
	li a7, 0xCBFB
	ecall
	li a0, 0xFF
	li a7, 93
	ecall
	
# 0 "" 2
 #NO_APP
	li	a5,1
	mv	a0,a5
	ld	s0,40(sp)
	addi	sp,sp,48
	jr	ra
	.size	main, .-main
	.ident	"GCC: (GNU) 8.2.0"
