# rust-amiga-emul

Just for fun!

There's no ambition to make this a fully functional Amiga emulator.

<https://www.nxp.com/docs/en/reference-manual/M68000PRM.pdf>

<http://www.faqs.org/faqs/motorola/68k-chips-faq/>

<https://www.amigacoding.com/index.php/Amiga_memory_map>

<http://oscomp.hu/depot/amiga_memory_map.html>

<https://retrocomputing.stackexchange.com/questions/1158/why-is-the-amiga-rom-at-a-high-memory-location-and-ram-in-low-memory>

<http://eab.abime.net/showthread.php?t=35282>
<http://eab.abime.net/attachment.php?attachmentid=16255&d=1206368942>

<https://wandel.ca/homepage/execdis/exec_disassembly.txt>

68000 instructions left to do: 40

instruction|68000|68008|68010|68020|68030|68040|68881/68882|68851|CPU32
-|:-:|:-:|:-:|:-:|:-:|:-:|:-:|:-:|:-:
ABCD (todo)|X|X|X|X|X|X|||X
ADD|X|X|X|X|X|X|||X
ADDA|X|X|X|X|X|X|||X
ADDI|X|X|X|X|X|X|||X
ADDQ|X|X|X|X|X|X|||X
ADDX|X|X|X|X|X|X|||X
AND|X|X|X|X|X|X|||X
ANDI|X|X|X|X|X|X|||X
ANDI to CCR (todo)|X|X|X|X|X|X|||X
ANDI to SR 1|X|X|X|X|X|X|||X
ASL,ASR (todo)|X|X|X|X|X|X|||X
Bcc|X|X|X|X|X|X|||X
BCHG (todo)|X|X|X|X|X|X|||X
BCLR|X|X|X|X|X|X|||X
BFCHG (todo)||||X|X|X|||
BFCLR (todo)||||X|X|X|||
BFEXTS (todo)||||X|X|X|||
BFEXTU (todo)||||X|X|X|||
BFFFO (todo)||||X|X|X|||
BFINS (todo)||||X|X|X|||
BFSET (todo)||||X|X|X|||
BFTST (todo)||||X|X|X|||
BGND (todo)|||||||||X
BKPT (todo)|||X|X|X|X|||X
BRA|X|X|X|X|X|X|||X
BSET|X|X|X|X|X|X|||X
BSR|X|X|X|X|X|X|||X
BTST|X|X|X|X|X|X|||X
CALLM (todo)||||X|||||
CAS,CAS2 (todo)||||X|X|X|||
CHK (todo)|X|X|X|X|X|X|||X
CHK2 (todo)||||X|X|X|||X
CINV 1 (todo)||||||X|||
CLR|X|X|X|X|X|X|||X
CMP|X|X|X|X|X|X|||X
CMPA|X|X|X|X|X|X|||X
CMPI|X|X|X|X|X|X|||X
CMPM|X|X|X|X|X|X|||X
CMP2 (todo)||||X|X|X||X
todo: cpBcc - CPUSH|||||||||
DBcc|X|X|X|X|X|X|||X
DIVS (todo)|X|X|X|X|X|X|||X
DIVSL (todo)||||X|X|X|||X
DIVU (todo)|X|X|X|X|X|X|||X
DIVUL (todo)||||X|X|X|||X
EOR (todo)|X|X|X|X|X|X|||X
EORI (todo)|X|X|X|X|X|X|||X
EORI to CCR (todo)|X|X|X|X|X|X|||X
EORI to SR 1 (todo)|X|X|X|X|X|X|||X
EXG|X|X|X|X|X|X|||X
EXT (todo)|X|X|X|X|X|X|||X
todo: EXTB - FTWOTOX|||||||||
ILLEGAL (todo)|X|X|X|X|X|X|||X
JMP|X|X|X|X|X|X|||X
JSR|X|X|X|X|X|X|||X
LEA|X|X|X|X|X|X|||X
LINK|X|X|X|X|X|X|||X
LPSTOP (todo)|||||||||
LSL,LSR|X|X|X|X|X|X|||X
MOVE|X|X|X|X|X|X|||X
MOVEA|X|X|X|X|X|X|||X
MOVE from CCR (todo)|||X|X|X|X|||X
MOVE to CCR (todo)|X|X|X|X|X|X|||X
MOVE from SR 1 (todo)|X 4|X 4|X|X|X|X|||X
MOVE to SR 1 (todo)|X|X|X|X|X|X|||X
MOVE USP 1 (todo)|X|X|X|X|X|X|||X
MOVE16 (todo)||||||X|||
MOVEC 1 (todo)|||X|X|X|X|||X
MOVEM|X|X|X|X|X|X|||X
MOVEP (todo)|X|X|X|X|X|X|||X
MOVEQ|X|X|X|X|X|X|||X
MOVES 1 (todo)|||X|X|X|X|||X
MULS (todo)|X|X|X|X|X|X|||X
MULU|X|X|X|X|X|X|||X
NBCD (todo)|X|X|X|X|X|X|||X
NEG (todo)|X|X|X|X|X|X|||X
NEGX (todo)|X|X|X|X|X|X|||X
NOP|X|X|X|X|X|X|||X
NOT|X|X|X|X|X|X|||X
OR (todo)|X|X|X|X|X|X|||X
ORI (todo)|X|X|X|X|X|X|||X
ORI to CCR (todo)|X|X|X|X|X|X|||X
ORI to SR 1 (todo)|X|X|X|X|X|X|||X
PACK (todo)||||X|X|X|||
PBcc 1 (todo)||||||||X|
PDBcc 1 (todo)||||||||X|
PEA|X|X|X|X|X|X|||X
todo: PFLUSH 1 to PVALID|||||||||
RESET 1 (todo)|X|X|X|X|X|X|||X
ROL,ROR (todo)|X|X|X|X|X|X|||X
ROXL,ROXR (todo)|X|X|X|X|X|X|||X
RTD (todo)|||X|X|X|X|||X
RTE 1|X|X|X|X|X|X|||X
RTM (todo)||||X|||||
RTR (todo)|X|X|X|X|X|X|||X
RTS|X|X|X|X|X|X|||X
SBCD (todo)|X|X|X|X|X|X|||X
Scc (todo)|X|X|X|X|X|X|||X
STOP 1 (todo)|X|X|X|X|X|X|||X
SUB|X|X|X|X|X|X|||X
SUBA|X|X|X|X|X|X|||X
SUBI|X|X|X|X|X|X|||X
SUBQ|X|X|X|X|X|X|||X
SUBX|X|X|X|X|X|X|||X
SWAP|X|X|X|X|X|X|||X
TAS (todo)|X|X|X|X|X|X|||X
TBLS, TBLSN (todo)|||||||||X
TBLU,TBLUN (todo)|||||||||X
TRAP (todo)|X|X|X|X|X|X|||X
TRAPcc (todo)||||X|X|X|||X
TRAPV (todo)|X|X|X|X|X|X|||X
TST|X|X|X|X|X|X|||X
UNLK|X|X|X|X|X|X|||X
UNPK (todo)||||X|X|X|||

1) Privileged (Supervisor) Instruction
2) Not applicable to MC68EC040 and MC68LC040
3) These instructions are software supported on the MC68040
4) This instruction is not privileged for the MC68000 and MC68008
5) Not applicable to MC68EC030