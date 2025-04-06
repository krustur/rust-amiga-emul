;---------------T---------T---------------------T------------------

LVOSupervisor		equ -30
LVOForbid		equ -132
LVOPermit		equ -138
LVOCacheClearU		equ -636
_LVOOpenLibrary		equ -552
_LVOCloseLibrary	equ -414

_LVOLoadView		equ -222
_LVOWaitBlit		equ -228
_LVOWaitTOF		equ -270
_LVOOwnBlitter		equ -456
_LVODisownBlitter	equ -462

_LVORethinkDisplay	equ -390

_LVOPutString		equ -948

gb_ActiView	equ 34	; ?
gb_copinit	equ 38	; ?

dmaconr	equ $002
intenar	equ $01c
cop1lc	equ $080
dmacon	equ $096
intena	equ $09a

true	equ $00	; can use beq for if true
false	equ $01	; can use bne for if not true

move_w_sr_absadr_l_instr	equ	$40f9	; move.w sr,($0).l
move_l_a7_to_absadr_l_instr	equ	$23cf	; move.l a7,($0).l
lea_l_0_pc_to_a7_instr	equ	$4ffafffe	; lea.l 0(pc),a7
jmp_absadr_l_instr	equ $4ef9	; jmp ($0).l

test_offs_name		equ	$00
test_offs_arrange_mem	equ	$04
test_offs_arrange_regs	equ	$08
test_offs_arrange_code	equ	$0c
test_offs_assert_mem	equ	$10
test_offs_assert_regs	equ	$14
test_offs_assert_code	equ	$18

WAIT_VBL	MACRO
.sync\@	btst	#0,$dff005
	beq.s	.sync\@
.sync2\@	btst	#0,$dff005
	bne.s	.sync2\@
	ENDM

	xref       _test_runner_run
start_of_all_code

_test_runner_run
test_runner_run
	;; Disable system
		
	movem.l	d0-d7/a0-a6,-(sp)
	move.l	sp,save_global_sp

	; Open graphics.library

	move.l	$4.w,a6

	lea	gfx_name(pc),a1
	moveq	#36,d0
	jsr	_LVOOpenLibrary(a6)
	tst.l	d0
	beq.w	exit
	move.l	d0,gfx_base

	; Open dos.library
	lea	dos_name(pc),a1
	moveq	#34,d0
	jsr	_LVOOpenLibrary(a6)
	tst.l	d0
	beq.w	exit
	move.l	d0,dos_base

	; Open intuition.library
	lea	intuition_name(pc),a1
	moveq	#36,d0
	jsr	_LVOOpenLibrary(a6)
	tst.l	d0
	beq.w	exit
	move.l	d0,intuition_base

	; Own blitter

	move.l	gfx_base,a6
	jsr	_LVOOwnBlitter(a6)
	jsr	_LVOWaitBlit(a6)
	
	; Forbid multitask
	
	move.l	$4.w,a6
	jsr	LVOForbid(a6)

	lea	$dff000,a5
	move.w	intenar(a5),d0
	move.w	d0,save_intena
	move.w	#$7fff,intena(a5)
	
	; Blank screen

	move.l	gfx_base,a6
	move.l	(gb_ActiView,a6),save_actiview
	move.l	(gb_copinit,a6),save_copinit

	sub.l	a1,a1
	jsr	_LVOLoadView(a6)
	jsr	_LVOWaitTOF(a6)
	jsr	_LVOWaitTOF(a6)

	; Disable DMA

	lea	$dff000,a5
	move.w	dmaconr(a5),d0
	move.w	d0,save_dmacon
	WAIT_VBL
	move.w	#$7fff,dmacon(a5)
	
	; Setup test-runner
	
	lea	test_suite,a0
	move.l	a0,next_test

test_loop
	; Fetch next test
	
	move.l	next_test,a0
	move.l	(a0)+,d0

	; Done when no more tests

	beq	done

	; Point to next test

	move.l	a0,next_test

	; Run test in supervisor mode

	move.l	d0,a0
	lea	run_test,a5
	move.l	$4,a6
	jsr	LVOSupervisor(a6)


	; Loop tests
	
	bra	test_loop

done
	lea	test_summary_total,a0
	move.l	test_count_tot,d0
	bsr	hex_to_str
	lea	test_summary_ok,a0
	move.l	test_count_ok,d0
	bsr	hex_to_str
	lea	test_summary_failed,a0
	move.l	test_count_fail,d0
	bsr	hex_to_str

	bsr	log_str_eol
	lea	test_summary_strz,a0
	bsr	log_strz

exit
	; Reenable system

	; Restore copper and DMA

	WAIT_VBL
	lea	$dff000,a6
	move.l	save_copinit,cop1lc(a6)
	move.w	save_dmacon,d0
	or.w	#$8000,d0
	move.w	d0,dmacon(a6)

	; Permit multitasking

	lea	$dff000,a6
	move.w	save_intena,d0
	or.w	#$8000,d0
	move.w	d0,intena(a6)
	
	move.l	$4.w,a6
	jsr	LVOPermit(a6)


	; Restore view

	move.l	gfx_base,a6
	move.l	save_actiview,a1
	jsr	_LVOLoadView(a6)
	jsr	_LVOWaitTOF(a6)
	jsr	_LVOWaitTOF(a6)
	
	move.l	intuition_base,a6
	jsr	_LVORethinkDisplay(a6)
	
	
	; Disown blitter

	move.l	gfx_base,a6
	jsr	_LVODisownBlitter(a6)

	; Print log

	move.l	dos_base,a6
	move.l	#log,d1
	jsr	_LVOPutString(a6)

	; Close libraries

	move.l	$4.w,a6
	move.l	gfx_base,d0
	beq.s	.no_gfx_base
	move.l	d0,a1
	jsr	_LVOCloseLibrary(a6)
	move.l	#-1,gfx_base
.no_gfx_base
	move.l	dos_base,d0
	beq.s	.no_dos_base
	move.l	d0,a1
	jsr	_LVOCloseLibrary(a6)
	move.l	#-1,dos_base
.no_dos_base
	move.l	intuition_base,d0
	beq.s	.no_intuition_base
	move.l	d0,a1
	jsr	_LVOCloseLibrary(a6)
	move.l	#-1,intuition_base
.no_intuition_base
	

	move.l	save_global_sp,sp
	movem.l	(sp)+,d0-d7/a0-a6

	; Set D-regs to show test counts in asm-one
	
;	move.l	test_count_tot,d0
;	move.l	test_count_ok,d1
;	move.l	test_count_fail,d2

	rts

save_global_sp	dc.l	$0
save_dmacon	dc.w	$0
save_intena	dc.w	$0
save_actiview	dc.l	$0
save_copinit	dc.l	$0

gfx_name	dc.b	"graphics.library",0
dos_name	dc.b	"dos.library",0
intuition_name	dc.b	"intuition.library",0
	align	0,2
	;even
gfx_base	dc.l	$0
dos_base	dc.l	$0
intuition_base	dc.l	$0

next_test	dc.l	$0

test_summary_strz
	dc.b	"Total tests: $"
test_summary_total
	dc.b	"xxxxxxxx",$a
	dc.b	"Ok tests: $"
test_summary_ok
	dc.b	"xxxxxxxx",$a
	dc.b	"Failed tests: "
test_summary_failed
	dc.b	"xxxxxxxx",$a,0
	align	0,2

;
; note: This is run in Supervisor mode so we can arrange and assert SR
;
; param:
;  A0 = test

run_test
	;; A5 = test
	
	move.l	a0,a5
	move.l	a5,current_test

	; increase total test count

	add.l	#1,test_count_tot

	; Backup&Safety: Memory areas

	move.l	test_offs_arrange_mem(a5),a0
	lea	mem_backup,a3
.backup_mem_loop
	move.l	(a0)+,d0	; d0 = mem length
	beq.s	.backup_mem_done
	move.l	(a0)+,a1	; a1 = mem address
	move.l	(a0)+,a2	; a2 = mem ptr (ignored)
	subq	#1,d0
.backup_mem_loop_inner
	move.b	(a1)+,(a3)+
	dbf	d0,.backup_mem_loop_inner
	bra.s	.backup_mem_loop
.backup_mem_done
	

	; Backup&Safety: Code

	move.l	test_offs_arrange_code(a5),a0
	move.l	(a0)+,d0	; d0 = code length
	move.l	(a0)+,a1	; a1 = code address
			; a0 = code ptr (ignored)

	lea	code_backup,a2
	addq.l	#3-1,d0	; 3 extra word for jmp back
.backup_code_loop
	move.w	(a1)+,(a2)+
	dbf	d0,.backup_code_loop	

		
	; Arrange: Memory areas

	move.l	test_offs_arrange_mem(a5),a0
	lea	mem_copy,a3
.arrange_mem_loop
	move.l	(a0)+,d0	; d0 = mem length
	beq.s	.arrange_mem_done
	move.l	(a0)+,a1	; a1 = mem address
	move.l	(a0)+,a2	; a2 = mem ptr
	subq	#1,d0
.arrange_mem_loop_inner
	move.b	(a2),(a3)+
	move.b	(a2)+,(a1)+
	dbf	d0,.arrange_mem_loop_inner
	bra.s	.arrange_mem_loop
.arrange_mem_done


	; Arrange: Code

	move.l	test_offs_arrange_code(a5),a0
	move.l	(a0)+,d0	; d0 = code length
	move.l	(a0)+,a1	; a1 = code address
			; a0 = code ptr

	move.l	a1,.test_jmp_address
	
	; code_copy can be used to view disassembly in asm-one
	lea	code_copy,a2
	subq	#1,d0
.copy_test_code_loop
	move.w	(a0),(a2)+
	move.w	(a0)+,(a1)+
	dbf	d0,.copy_test_code_loop	

	; now we generate some post-test-code code to collect sr and pc
	; and jmp back to the test runner code

	; move.w    sr,collected_sr
	move.w	#move_w_sr_absadr_l_instr,(a1)+
	move.w	#move_w_sr_absadr_l_instr,(a2)+
	move.l	#collected_sr,(a1)+
	move.l	#collected_sr,(a2)+

	; move.l a7,collected_regs+$3c
	move.w	#move_l_a7_to_absadr_l_instr,(a1)+
	move.w	#move_l_a7_to_absadr_l_instr,(a2)+
	move.l #collected_regs+$3c,(a1)+
	move.l #collected_regs+$3c,(a2)+

	; lea 0(pc),a7
	; 2 * word+long instructions above that should be subtracted from collected_pc later
	move.l #lea_l_0_pc_to_a7_instr,(a1)+
	move.l #lea_l_0_pc_to_a7_instr,(a2)+

	; move.l a7,collected_pc
	move.w	#move_l_a7_to_absadr_l_instr,(a1)+
	move.w	#move_l_a7_to_absadr_l_instr,(a2)+
	move.l #collected_pc,(a1)+
	move.l #collected_pc,(a2)+

	; jmp .return_here_after_test
	move.w	#jmp_absadr_l_instr,(a1)+
	move.w	#jmp_absadr_l_instr,(a2)+
	move.l	#.return_here_after_test,(a1)
	move.l	#.return_here_after_test,(a2)

	; Arrange: Clear caches

	move.l	$4.w,a6
	jsr	LVOCacheClearU(a6)

	; Backup&Safety: Stack Pointer

	move.l	sp,sp_backup

	; Arrange: A-/D-regs + SR

	move.l	test_offs_arrange_regs(a5),a7

	; SR

	move.w	64(a7),d0
	move.w	sr,d1
	and.w	#$ffe0,d1
	or.w	d1,d0
	move.w	d0,arrange_sr

	; Regs
	
	movem.l	(a7)+,d0-d7/a0-a6
	move.l	(a7),a7
	move.w	arrange_sr,sr

	; Act: Run test!
	
	; this is a jmp (xxx).l instruction to the
	; test code
	dc.w	jmp_absadr_l_instr
.test_jmp_address
	dc.l	$12345678

	; the test will jmp back to this address 
	; after test code is run
.return_here_after_test

	; Act: Collect regs
	
	and.w	#$001f,collected_sr
	sub.l	#12,collected_pc	; adjust collected_pc as stated above ...
	move.l	d0,collected_regs+$00
	move.l	d1,collected_regs+$04
	move.l	d2,collected_regs+$08
	move.l	d3,collected_regs+$0c
	move.l	d4,collected_regs+$10
	move.l	d5,collected_regs+$14
	move.l	d6,collected_regs+$18
	move.l	d7,collected_regs+$1c
	move.l	a0,collected_regs+$20
	move.l	a1,collected_regs+$24
	move.l	a2,collected_regs+$28
	move.l	a3,collected_regs+$2c
	move.l	a4,collected_regs+$30
	move.l	a5,collected_regs+$34
	move.l	a6,collected_regs+$38
	; move.l	a7,collected_regs+$3c

	; Act: Collect mem

	move.l	current_test,a5
	move.l	test_offs_assert_mem(a5),a0
	lea	collected_mem,a3
.collect_mem_loop
	move.l	(a0)+,d0	; d0 = mem length
	beq.s	.collect_mem_done
	move.l	(a0)+,a1	; a1 = mem address
	move.l	(a0)+,a2	; a2 = mem ptr (ignored)
	subq	#1,d0
.collect_mem_loop_inner
	move.b	(a1)+,(a3)+
	dbf	d0,.collect_mem_loop_inner
	bra.s	.collect_mem_loop
.collect_mem_done

	; Restore: SP

	move.l	sp_backup,sp

	; Restore: Code

	move.l	current_test,a5
	move.l	test_offs_arrange_code(a5),a0
	move.l	(a0)+,d0	; d0 = code length
	move.l	(a0)+,a1	; a1 = code address
			; a0 = code ptr

	lea	code_backup,a2
	addq.l	#3-1,d0
.restore_code_loop 
	move.w	(a2)+,(a1)+
	dbf	d0,.restore_code_loop	

	; Restore: Memory

	move.l	test_offs_arrange_mem(a5),a0
	lea	mem_backup,a3
.restore_mem_loop
	move.l	(a0)+,d0	; d0 = mem length
	beq.s	.restore_mem_done
	move.l	(a0)+,a1	; a1 = mem address
	move.l	(a0)+,a2	; a2 = mem ptr (ignored)
	subq	#1,d0
.restore_mem_loop_inner
	move.b	(a3)+,(a1)+
	dbf	d0,.restore_mem_loop_inner
	bra.s	.restore_mem_loop
.restore_mem_done
	

	; Restore: Clear caches
	move.l	$4.w,a6
	jsr	LVOCacheClearU(a6)
	

	; Assert: Check if ok/fail


	; Code

	move.l	current_test,a5
	move.l	test_offs_arrange_code(a5),a0
	move.l	(a0),d0
	add.l	#8,a0
	move.l	test_offs_assert_code(a5),a1
	subq	#1,d0
	move.b	#false,.code_failed
.check_code_loop
	move.w	(a0)+,d1
	move.w	(a1)+,d2
	cmp.w	d1,d2
	bne.w	.code_fail
	dbf	d0,.check_code_loop	

	; Mem

	move.l	test_offs_assert_mem(a5),a0
	lea	collected_mem,a3
.check_mem_loop
	move.l	(a0)+,d0	; d0 = mem length
	beq.s	.check_mem_done
	move.l	(a0)+,a1	; a1 = mem address (ignored)
	move.l	(a0)+,a2	; a2 = mem ptr
	subq	#1,d0
	move.b	#false,.mem_failed
.check_mem_loop_inner
	move.b	(a2)+,d1
	move.b	(a3)+,d2
	cmp.b	d1,d2
	bne.w	.mem_fail
	dbf	d0,.check_mem_loop_inner
	bra.s	.check_mem_loop
.check_mem_done

	; Registers

	move.l	test_offs_assert_regs(a5),a0
	
	move.l	collected_regs+$00,d0	; D0
	cmp.l	$00(a0),d0
	bne.w	.fail
	move.l	collected_regs+$04,d0	; D1
	cmp.l	$04(a0),d0
	bne.w	.fail
	move.l	collected_regs+$08,d0	; D2
	cmp.l	$08(a0),d0
	bne.w	.fail
	move.l	collected_regs+$0c,d0	; D3
	cmp.l	$0c(a0),d0
	bne.w	.fail
	move.l	collected_regs+$10,d0	; D4
	cmp.l	$10(a0),d0
	bne.w	.fail
	move.l	collected_regs+$14,d0	; D5
	cmp.l	$14(a0),d0
	bne.w	.fail
	move.l	collected_regs+$18,d0	; D6
	cmp.l	$18(a0),d0
	bne.w	.fail
	move.l	collected_regs+$1c,d0	; D7
	cmp.l	$1c(a0),d0
	bne.w	.fail
	move.l	collected_regs+$20,d0	; A0
	cmp.l	$20(a0),d0
	bne.w	.fail
	move.l	collected_regs+$24,d0	; A1
	cmp.l	$24(a0),d0
	bne.w	.fail
	move.l	collected_regs+$28,d0	; A2
	cmp.l	$28(a0),d0
	bne.w	.fail
	move.l	collected_regs+$2c,d0	; A3
	cmp.l	$2c(a0),d0
	bne.s	.fail
	move.l	collected_regs+$30,d0	; A4
	cmp.l	$30(a0),d0
	bne.s	.fail
	move.l	collected_regs+$34,d0	; A5
	cmp.l	$34(a0),d0
	bne.s	.fail
	move.l	collected_regs+$38,d0	; A6
	cmp.l	$38(a0),d0
	bne.s	.fail
	move.l	collected_regs+$3c,d0	; A7
	cmp.l	$3c(a0),d0
	bne.s	.fail
	move.l	collected_pc,d0	; PC
	cmp.l	$40(a0),d0
	bne.s	.fail
	move.w	collected_sr,d0	; SR
	cmp.w	$44(a0),d0
	bne.s	.fail


	; All passed - test is ok!
	
	add.l	#1,test_count_ok

    ; This code logs "  ok ...."

	; bsr	log_str_ok
	; move.l	(a5),a0
	; bsr	log_strz
	; bsr	log_str_eol

	rte

	; fail!
.code_fail
	move.b	#true,.code_failed
	bra.s	.fail

.mem_fail
	move.b	#true,.mem_failed
	
.fail
	add.l	#1,test_count_fail

	; Assert: Log failed test

	move.l	current_test,a5
	bsr	log_str_fail
	move.l	(a5),a0
	bsr	log_strz
	bsr	log_str_eol

	; Assert: Log failed mem


	move.b	.mem_failed,d0
	bne.s	.no_mem_fail_log

	move.l	current_test,a5
	move.l	test_offs_assert_mem(a5),a4
	lea	collected_mem,a3
.mem_fail_log_loop
	move.l	(a4)+,d1
	beq.s	.mem_fail_log_done

	move.l	(a4)+,d0
	lea	.fail_mem_strza,a0
	bsr.w	hex_to_str
	
	lea	.fail_mem_strz,a0
	bsr	log_strz

	move.l	d1,d0
	move.l	(a4)+,a0

	bsr	log_hex_dump
	bsr	log_str_eol
	
	lea	.fail_memb_strz,a0
	bsr	log_strz


	move.l	d1,d0
	move.l	a3,a0
	bsr	log_hex_dump
	move.l	a0,a3

	bsr	log_str_eol
	bra.s	.mem_fail_log_loop

.mem_fail_log_done
.no_mem_fail_log

	; Assert: Log failed code

	move.b	.code_failed,d0
	bne.s	.no_code_fail_log

	lea	.fail_code_strz,a0
	bsr	log_strz

	move.l	current_test,a5
	move.l	test_offs_arrange_code(a5),a0
	move.l	(a0),d0
	asl.l	#1,d0
	add.l	#8,a0
	bsr	log_hex_dump
	bsr	log_str_eol
	
	lea	.fail_codeb_strz,a0
	bsr	log_strz

	move.l	current_test,a5
	move.l	test_offs_arrange_code(a5),a0
	move.l	(a0),d0
	asl.l	#1,d0
	move.l	test_offs_assert_code(a5),a0
	bsr	log_hex_dump

	bsr	log_str_eol
	
.no_code_fail_log

	; Assert: Log fail details - regs

	move.l	test_offs_assert_regs(a5),a4
	
	move.l	collected_regs+$00,d0	; D0
	move.l	$00(a4),d1
	move.w	#"D0",d2
	bsr.w	.fail_reg_details
	move.l	collected_regs+$04,d0	; D1
	move.l	$04(a4),d1
	move.w	#"D1",d2
	bsr.w	.fail_reg_details
	move.l	collected_regs+$08,d0	; D2
	move.l	$08(a4),d1
	move.w	#"D2",d2
	bsr.w	.fail_reg_details
	move.l	collected_regs+$0c,d0	; D3
	move.l	$0c(a4),d1
	move.w	#"D3",d2
	bsr.w	.fail_reg_details
	move.l	collected_regs+$10,d0	; D4
	move.l	$10(a4),d1
	move.w	#"D4",d2
	bsr.w	.fail_reg_details
	move.l	collected_regs+$14,d0	; D5
	move.l	$14(a4),d1
	move.w	#"D5",d2
	bsr.w	.fail_reg_details
	move.l	collected_regs+$18,d0	; D6
	move.l	$18(a4),d1
	move.w	#"D6",d2
	bsr.w	.fail_reg_details
	move.l	collected_regs+$1c,d0	; D7
	move.l	$1c(a4),d1
	move.w	#"D7",d2
	bsr.w	.fail_reg_details
	move.l	collected_regs+$20,d0	; A0
	move.l	$20(a4),d1
	move.w	#"A0",d2
	bsr.w	.fail_reg_details
	move.l	collected_regs+$24,d0	; A1
	move.l	$24(a4),d1
	move.w	#"A1",d2
	bsr.w	.fail_reg_details
	move.l	collected_regs+$28,d0	; A2
	move.l	$28(a4),d1
	move.w	#"A2",d2
	bsr.s	.fail_reg_details
	move.l	collected_regs+$2c,d0	; A3
	move.l	$2c(a4),d1
	move.w	#"A3",d2
	bsr.s	.fail_reg_details
	move.l	collected_regs+$30,d0	; A4
	move.l	$30(a4),d1
	move.w	#"A4",d2
	bsr.s	.fail_reg_details
	move.l	collected_regs+$34,d0	; A5
	move.l	$34(a4),d1
	move.w	#"A5",d2
	bsr.s	.fail_reg_details
	move.l	collected_regs+$38,d0	; A6
	move.l	$38(a4),d1
	move.w	#"A6",d2
	bsr.s	.fail_reg_details
	move.l	collected_regs+$3c,d0	; A7
	move.l	$3c(a4),d1
	move.w	#"A7",d2
	bsr.s	.fail_reg_details
	move.l	collected_pc,d0	; PC
	move.l	$40(a4),d1
	move.w	#"PC",d2
	bsr.s	.fail_reg_details
	moveq	#0,d0	; SR - is word, so clear upper word of d0 and d1
	moveq	#0,d1
	move.w	collected_sr,d0
	move.w	$44(a4),d1
	move.w	#"SR",d2
	bsr.s	.fail_reg_details

	rte

.fail_reg_details
	; d0=was
	; d1=expected
	; d2=reg ascii (word)
	cmp.l	d0,d1
	beq.s	.fail_reg_ok
	move.w	d2,.fail_reg_strza
	lea	.fail_reg_strzc,a0
	bsr.w	hex_to_str
	move.l	d1,d0
	lea	.fail_reg_strzb,a0
	bsr.w	hex_to_str
	lea	.fail_reg_strz,a0
	bsr.w	log_strz
.fail_reg_ok
	rts

.fail_reg_strz	dc.b	"     "
.fail_reg_strza dc.b	"XX: expected $"
.fail_reg_strzb	dc.b	"XXXXXXXX - was $"
.fail_reg_strzc	dc.b	"XXXXXXXX",$a,0

.fail_code_strz	dc.b	"     Code:",$a
	dc.b	"      expected ",0
.fail_codeb_strz dc.b	"      was      ",0

.fail_mem_strz	dc.b	"     Mem: $"
.fail_mem_strza	dc.b	"XXXXXXXX",$a
	dc.b	"      expected ",0
.fail_memb_strz dc.b	"      was      ",0

.code_failed	dc.b	0
.mem_failed	dc.b	0
		align	0,2

current_test	dc.l	$00000000
test_count_tot	dc.l	$00000000
test_count_ok	dc.l	$00000000
test_count_fail	dc.l	$00000000

arrange_sr	dc.w	$0000
collected_sr	dc.w	$0000
collected_pc	dc.l	$00000000
collected_regs	blk.l	16,$00000000
collected_mem	blk.b	2048,$00

code_backup	blk.b	512,$ff
code_copy	blk.b	512,$ff
mem_backup	blk.b	2048,$ff
mem_copy	blk.b	2048,$ff
sp_backup	dc.l	$00000000

; Logger functions

log_strz
	movem.l	d0/a1-a2,-(sp)
	;move.l	d0,log_save_d0
	;move.l	a1,log_save_a1
	;move.l	a2,log_save_a2

	move.l	log_current_ptr,a1
	lea.l	log_buffer_end,a2
.loop
	move.b	(a0)+,d0
	beq.s	.done

	cmp.l	a1,a2
	beq.s	.overflow

	move.b	d0,(a1)+
	bra.s	.loop

.overflow
	bsr.w	log_overflow
;	bra	.done

.done
	move.l	a1,log_current_ptr

	;move.l	log_save_a2,a2
	;move.l	log_save_a1,a1
	;move.l  log_save_d0,d0
	movem.l	(sp)+,d0/a1-a2
	rts

log_str_ok
	move.l	a0,-(sp)
	lea	.str,a0
	bsr	log_strz
	move.l	(sp)+,a0
	rts

.str	dc.b	"  ok ",0
	even
	
log_str_fail
	move.l	a0,-(sp)
	lea	.str,a0
	bsr	log_strz
	move.l	(sp)+,a0
	rts

.str	dc.b	"fail ",0
	even

log_str_eol
	move.l	a0,-(sp)
	lea	.str,a0
	bsr	log_strz
	move.l	(sp)+,a0
	rts

.str	dc.b	$0a,0
	even

log_hex_dump
	movem.l	d1-d3/a1-a3,-(sp)

	subq	#1,d0

	move.l	log_current_ptr,a1
	lea.l	log_buffer_end,a2
	lea	hex_char_table,a3

	moveq	#0,d1
	moveq	#0,d2
.loop
	cmp.l	a1,a2
	beq.s	.overflow

	move.b	(a0)+,d1

	move.b	d1,d2
	lsr.b	#4,d2
	move.b	(a3,d2),(a1)+

	cmp.l	a1,a2
	beq.s	.overflow


	and.b	#$0f,d1
	move.b	(a3,d1),(a1)+

	cmp.l	a1,a2
	beq.s	.overflow

	move.b	#" ",(a1)+
	
	dbf	d0,.loop
	bra.s	.done

.overflow
	bsr.s	log_overflow

.done
	move.l	a1,log_current_ptr

	movem.l	(sp)+,d1-d3/a1-a3
	rts
	
log_overflow
	move.l	#"log ",log
	move.l	#"over",log+4
	move.l	#"flow",log+8
	rts

hex_to_str
	movem.l	d1-d2/a1,-(sp)
	lea	hex_char_table,a1
	add.l	#8,a0
	moveq	#0,d1
	moveq	#8-1,d2
.loop
	move.b	d0,d1
	and.b	#$0f,d1

	move.b	(a1,d1),-(a0)

	ror.l	#4,d0
	dbra	d2,.loop
	movem.l	(sp)+,d1-d2/a1
	rts
	
hex_char_table
	dc.b	"0123456789ABCDEF"

log_current_ptr	dc.l log
;log_save_d0	dc.l $00000000
;log_save_a1	dc.l $00000000	

log	blk.b 1000*100,$00
log_buffer_end
	; final zero ending here in case we 
	; fill the entire log
	dc.b	$00
	even

	; include test suite
	incdir	"tests/"
	include	"test_suite.s"

	; original test suite
;	include	"dev:github/rust-amiga-emul-ami-test-runner/test_suite.s"


end_of_all_code
