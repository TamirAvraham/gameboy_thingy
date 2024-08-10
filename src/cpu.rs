use crate::memory::Memory;
use crate::register::Flags::{C, H, N, Z};
use crate::register::Registers;

struct Clock {
    m: u128,
    t: u128,
}
struct Cpu {
    clock: Clock,
    memory: Memory,
    registers: Registers,
}
enum Register8bit {
    A,
    B,
    C,
    D,
    E,
    H,
    L,
}
enum Register16bit {
    BC,
    DE,
    HL,
    SP,
}

impl Cpu {
    #[inline]
    fn get_register_from_register16bit(&mut self, register16bit: Register16bit) -> u16 {
        match register16bit {
            Register16bit::BC => self.registers.get_bc(),
            Register16bit::DE => self.registers.get_de(),
            Register16bit::HL => self.registers.get_hl(),
            Register16bit::SP => self.registers.sp,
        }
    }
    #[inline]
    fn load_16bit_value_into_register(&mut self, register16bit: Register16bit, value: u16) {
        match register16bit {
            Register16bit::BC => self.registers.write_bc(value),
            Register16bit::DE => self.registers.write_de(value),
            Register16bit::HL => self.registers.write_hl(value),
            Register16bit::SP => self.registers.write_hl(value),
        };
        self.clock.m += 10;
    }
    #[inline]
    fn load_8bit_value_into_register(&mut self, register8bit: Register8bit, value: u8) {
        match register8bit {
            Register8bit::A => self.registers.a = value,
            Register8bit::B => self.registers.b = value,
            Register8bit::C => self.registers.c = value,
            Register8bit::D => self.registers.d = value,
            Register8bit::E => self.registers.e = value,
            Register8bit::H => self.registers.h = value,
            Register8bit::L => self.registers.l = value,
        };
        self.clock.m += 7;
    }
    #[inline(always)]
    fn inc_u8_refm(v: &mut u8) -> u8 {
        *v = v.wrapping_add(1);
        *v
    }
    #[inline]
    fn inc_8bit_register_set_flags(registers: &mut Registers, v: u8) {
        registers
            .flag(Z, v == 0)
            .flag(N, false)
            .flag(H, (v & 0x0F) + 1 > 0x0F);
    }
    #[inline(always)]
    fn get_register_refm_from_register_8bit(&mut self, register8bit: Register8bit) -> &mut u8 {
        match register8bit {
            Register8bit::A => &mut self.registers.a,
            Register8bit::B => &mut self.registers.b,
            Register8bit::C => &mut self.registers.c,
            Register8bit::D => &mut self.registers.d,
            Register8bit::E => &mut self.registers.e,
            Register8bit::H => &mut self.registers.h,
            Register8bit::L => &mut self.registers.l,
        }
    }
    #[inline]
    fn inc_8bit_register(&mut self, register8bit: Register8bit) {
        let cache = Self::inc_u8_refm(self.get_register_refm_from_register_8bit(register8bit));
        Self::inc_8bit_register_set_flags(&mut self.registers, cache);
        self.clock.m += 4;
    }
    #[inline]
    fn inc_16bit_register(&mut self, register16bit: Register16bit) {
        match register16bit {
            Register16bit::BC => self
                .registers
                .write_bc(self.registers.get_bc().wrapping_add(1)),
            Register16bit::DE => self
                .registers
                .write_de(self.registers.get_de().wrapping_add(1)),
            Register16bit::HL => self
                .registers
                .write_hl(self.registers.get_hl().wrapping_add(1)),
            Register16bit::SP => {
                self.registers.sp = self.registers.sp.wrapping_add(1);
                self.registers.sp
            }
        };
        self.clock.m += 8;
    }
    #[inline]
    fn dec_8bit_register(&mut self, register8bit: Register8bit) {
        let register = self.get_register_refm_from_register_8bit(register8bit);
        let new_register = register.wrapping_sub(1);
        *register = new_register;
        self.registers
            .flag(Z, new_register == 0)
            .flag(N, true)
            .flag(H, (new_register & 0x0F) == 0);
        self.clock.m += 4;
    }
    #[inline]
    fn dec_16bit_register(&mut self, register16bit: Register16bit) {
        match register16bit {
            Register16bit::BC => self
                .registers
                .write_bc(self.registers.get_bc().wrapping_sub(1)),
            Register16bit::DE => self
                .registers
                .write_de(self.registers.get_de().wrapping_sub(1)),
            Register16bit::HL => self
                .registers
                .write_hl(self.registers.get_hl().wrapping_sub(1)),
    Register16bit::SP => {
                self.registers.sp = self.registers.sp.wrapping_sub(1);
                self.registers.sp
            }
        };
        self.clock.m += 6;
    }
    #[inline(always)]
    fn use_carry(&self, use_carry: bool) -> u8 {
        if use_carry && self.registers.get_flag(C) {
            1
        } else {
            0
        }
    }
    #[inline]
    fn add_8bit_value(&mut self, value: u8, use_carry: bool) {
        let carry = self.use_carry(use_carry);
        let a = self.registers.a;
        let new_a = a.wrapping_add(value).wrapping_add(carry);

        self.registers
            .flag(Z, new_a == 0)
            .flag(N, false)
            .flag(H, (a & 0xF) + (value & 0xF) + carry > 0xF)
            .flag(C, (a as u16) + (value as u16) + (carry as u16) > 0xFF);
        self.registers.a = new_a;
        self.clock.m += 4;
    }
    #[inline]
    fn add_16bit_value(&mut self, value: u16) {
        let hl = self.registers.get_hl();
        let new_hl = hl.wrapping_add(value);
        self.registers
            .flag(H, (hl & 0x07FF) + (value & 0x7FF) > 0x07FF)
            .flag(C, hl > 0xFFFF - value)
            .flag(N, false);
        self.registers.write_hl(new_hl);
        self.clock.m += 8;
    }
    #[inline]
    fn sub_8bit_value(&mut self, value: u8, use_carry: bool) {
        let carry = self.use_carry(use_carry);
        let a = self.registers.a;
        let new_a = a.wrapping_sub(value).wrapping_sub(carry);
        self.registers
            .flag(Z, new_a == 0)
            .flag(H, (a & 0x0F) < (value & 0x0F) + carry)
            .flag(C, (a as u16) < (value as u16) + (carry as u16))
            .flag(N, true);
        self.registers.a = new_a;
        self.clock.m += 4;
    }
    fn push(&mut self, v: u16) {
        self.registers.sp = self.registers.sp.wrapping_sub(2);
        self.memory.write_word(self.registers.sp, v);
        self.clock.m+=16;
    }
    fn pop(&mut self)->u16{
        let stack_data=self.memory.read_word(self.registers.sp);
        self.registers.sp = self.registers.sp.wrapping_add(2);
        stack_data
    }
    fn pop_register16bit(&mut self, register16bit: Register16bit) {
        let stack_data=self.pop();
        match register16bit {
            Register16bit::BC => self
                .registers
                .write_bc(stack_data),
            Register16bit::DE => self
                .registers
                .write_de(stack_data),
            Register16bit::HL => self
                .registers
                .write_hl(stack_data),
            Register16bit::SP => {
                panic!("cant pop onto sp")
            }
        };
        self.clock.m+=12;
    }

    fn pop_af(&mut self) {
        self.registers.write_af(self.pop());
        self.clock.m += 12;
    }
    fn call(&mut self,label:u16,cond:bool){
        if cond {self.push(self.registers.pc.wrapping_add(3));
        self.registers.pc=label;
        self.clock.m+=17}else {self.clock.m+=10}
    }
    //for some fucking reason pure ret does 10 cycles instead of 11 like the others
    //so there ia now a new function named pure_ret
    #[inline(always)]
    fn pure_ret(&mut self){
        self.ret(true);
        self.clock.m-=1;
    }
    fn ret(&mut self,cond:bool){
        if cond {
            self.registers.pc=self.pop();
            self.clock.m+=11;
        } else {
            self.clock.m+=5;
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::register::Registers;
    const TEST_SP_ADDR:u16=0x000F;
    fn create_cpu() -> Cpu {
        Cpu {
            clock: Clock { m: 0, t: 0 },
            registers: Registers::default(),
            memory: Memory::new(),
        }
    }

    #[test]
    fn test_get_register_from_register16bit() {
        let mut cpu = create_cpu();
        cpu.registers.write_bc(0x1234);
        cpu.registers.write_de(0x5678);
        cpu.registers.write_hl(0x9ABC);

        assert_eq!(
            cpu.get_register_from_register16bit(Register16bit::BC),
            0x1234
        );
        assert_eq!(
            cpu.get_register_from_register16bit(Register16bit::DE),
            0x5678
        );
        assert_eq!(
            cpu.get_register_from_register16bit(Register16bit::HL),
            0x9ABC
        );
    }

    #[test]
    fn test_load_16bit_value_into_register() {
        let mut cpu = create_cpu();

        cpu.load_16bit_value_into_register(Register16bit::BC, 0x1111);
        assert_eq!(cpu.registers.get_bc(), 0x1111);
        assert_eq!(cpu.clock.m, 10);

        cpu.load_16bit_value_into_register(Register16bit::DE, 0x2222);
        assert_eq!(cpu.registers.get_de(), 0x2222);
        assert_eq!(cpu.clock.m, 20);

        cpu.load_16bit_value_into_register(Register16bit::HL, 0x3333);
        assert_eq!(cpu.registers.get_hl(), 0x3333);
        assert_eq!(cpu.clock.m, 30);
    }

    #[test]
    fn test_load_8bit_value_into_register() {
        let mut cpu = create_cpu();

        cpu.load_8bit_value_into_register(Register8bit::A, 0xAA);
        assert_eq!(cpu.registers.a, 0xAA);
        assert_eq!(cpu.clock.m, 7);

        cpu.load_8bit_value_into_register(Register8bit::B, 0xBB);
        assert_eq!(cpu.registers.b, 0xBB);
        assert_eq!(cpu.clock.m, 14);
    }

    #[test]
    fn test_inc_u8_refm() {
        let mut val = 0xFE;
        assert_eq!(Cpu::inc_u8_refm(&mut val), 0xFF);
        assert_eq!(val, 0xFF);

        assert_eq!(Cpu::inc_u8_refm(&mut val), 0x00);
        assert_eq!(val, 0x00);
    }

    #[test]
    fn test_inc_8bit_register() {
        let mut cpu = create_cpu();
        cpu.registers.a = 0xFE;
        cpu.inc_8bit_register(Register8bit::A);
        assert_eq!(cpu.registers.a, 0xFF);
        assert_eq!(cpu.registers.get_flag(Z), false);
        assert_eq!(cpu.registers.get_flag(H), true);
        assert_eq!(cpu.clock.m, 4);
    }

    #[test]
    fn test_inc_16bit_register() {
        let mut cpu = create_cpu();
        cpu.registers.write_bc(0xFFFE);
        cpu.inc_16bit_register(Register16bit::BC);
        assert_eq!(cpu.registers.get_bc(), 0xFFFF);
        assert_eq!(cpu.clock.m, 8);
    }

    #[test]
    fn test_dec_8bit_register() {
        let mut cpu = create_cpu();
        cpu.registers.a = 0x01;
        cpu.dec_8bit_register(Register8bit::A);
        assert_eq!(cpu.registers.a, 0x00);
        assert_eq!(cpu.registers.get_flag(Z), true);
        assert_eq!(cpu.registers.get_flag(N), true);
        assert_eq!(cpu.registers.get_flag(H), true);
        assert_eq!(cpu.clock.m, 4);
    }

    #[test]
    fn test_dec_16bit_register() {
        let mut cpu = create_cpu();
        cpu.registers.write_bc(0x0001);
        cpu.dec_16bit_register(Register16bit::BC);
        assert_eq!(cpu.registers.get_bc(), 0);
        assert_eq!(cpu.clock.m, 6);
    }

    #[test]
    fn test_use_carry() {
        let mut cpu = create_cpu();
        cpu.registers.flag(C, true);
        assert_eq!(cpu.use_carry(true), 1);
        assert_eq!(cpu.use_carry(false), 0);
    }

    #[test]
    fn test_add_8bit_value() {
        let mut cpu = create_cpu();
        cpu.registers.a = 0x50;
        cpu.add_8bit_value(0x10, false);
        assert_eq!(cpu.registers.a, 0x60);
        assert_eq!(cpu.registers.get_flag(Z), false);
        assert_eq!(cpu.registers.get_flag(N), false);
        assert_eq!(cpu.registers.get_flag(H), false);
        assert_eq!(cpu.registers.get_flag(C), false);
        assert_eq!(cpu.clock.m, 4);
    }

    #[test]
    fn test_add_16bit_value() {
        let mut cpu = create_cpu();
        cpu.registers.write_hl(0x1000);
        cpu.add_16bit_value(0x0100);
        assert_eq!(cpu.registers.get_hl(), 0x1100);
        assert_eq!(cpu.registers.get_flag(H), false);
        assert_eq!(cpu.registers.get_flag(C), false);
        assert_eq!(cpu.registers.get_flag(N), false);
        assert_eq!(cpu.clock.m, 8);
    }

    #[test]
    fn test_sub_8bit_value() {
        let mut cpu = create_cpu();
        cpu.registers.a = 0x60;
        cpu.sub_8bit_value(0x10, false);
        assert_eq!(cpu.registers.a, 0x50);
        assert_eq!(cpu.registers.get_flag(Z), false);
        assert_eq!(cpu.registers.get_flag(N), true);
        assert_eq!(cpu.registers.get_flag(H), false);
        assert_eq!(cpu.registers.get_flag(C), false);
        assert_eq!(cpu.clock.m, 4);
    }
    #[test]
    fn test_push() {
        let mut cpu = create_cpu();
        cpu.registers.sp = TEST_SP_ADDR.wrapping_add(2);
        cpu.push(0x1234);

        assert_eq!(cpu.memory.read_word(TEST_SP_ADDR), 0x1234);
        assert_eq!(cpu.registers.sp, TEST_SP_ADDR);
        assert_eq!(cpu.clock.m, 16);
    }

    #[test]
    fn test_pop_bc() {
        let mut cpu = create_cpu();
        cpu.registers.sp = TEST_SP_ADDR;
        cpu.memory.write_word(TEST_SP_ADDR, 0x1234);
        cpu.pop(Register16bit::BC);

        assert_eq!(cpu.registers.get_bc(), 0x1234);
        assert_eq!(cpu.registers.sp, TEST_SP_ADDR.wrapping_add(2));
        assert_eq!(cpu.clock.m, 12);
    }

    #[test]
    fn test_pop_de() {
        let mut cpu = create_cpu();
        cpu.registers.sp = TEST_SP_ADDR;
        cpu.memory.write_word(TEST_SP_ADDR, 0x5678);
        cpu.pop(Register16bit::DE);

        assert_eq!(cpu.registers.get_de(), 0x5678);
        assert_eq!(cpu.registers.sp, TEST_SP_ADDR.wrapping_add(2));
        assert_eq!(cpu.clock.m, 12);
    }

    #[test]
    fn test_pop_hl() {
        let mut cpu = create_cpu();
        cpu.registers.sp = TEST_SP_ADDR;
        cpu.memory.write_word(TEST_SP_ADDR, 0x9ABC);
        cpu.pop(Register16bit::HL);

        assert_eq!(cpu.registers.get_hl(), 0x9ABC);
        assert_eq!(cpu.registers.sp, TEST_SP_ADDR.wrapping_add(2));
        assert_eq!(cpu.clock.m, 12);
    }

    #[test]
    #[should_panic(expected = "cant pop onto sp")]
    fn test_pop_sp_should_panic() {
        let mut cpu = create_cpu();
        cpu.registers.sp = TEST_SP_ADDR;
        cpu.memory.write_word(TEST_SP_ADDR, 0x1234);
        cpu.pop(Register16bit::SP);
    }

    #[test]
    fn test_pop_af() {
        let mut cpu = create_cpu();
        cpu.registers.sp = TEST_SP_ADDR;
        cpu.memory.write_word(TEST_SP_ADDR, 0x5678);
        cpu.pop_af();
        assert_eq!(cpu.registers.a,0x56);

        assert_eq!(cpu.registers.get_af(), 0x5678); // Assuming you have a get_af() method
        assert_eq!(cpu.registers.sp, TEST_SP_ADDR.wrapping_add(2));
        assert_eq!(cpu.clock.m, 12);
    }
}
