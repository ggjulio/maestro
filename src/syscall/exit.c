#include <syscall/syscall.h>
#include <idt/idt.h>
#include <pic/pic.h>

__attribute__((noreturn))
sys_ret_t sys_exit(process_t *process, const regs_t *registers)
{
	process_exit(process, registers->ebx);
	pic_EOI(0x80);
	STI();
	asm("int $0x20");
	while(1)
		;
}