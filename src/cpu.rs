use crate::register::Flags::{H, N, Z};
use crate::register::Registers;

struct Clock {
    m: u128,
    t: u128,
}
struct Cpu {
    clock: Clock,
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
}

impl Cpu {
    #[inline]
    fn get_register_from_register16bit(&mut self, register16bit: Register16bit) -> u16 {
        match register16bit {
            Register16bit::BC => self.registers.get_bc(),
            Register16bit::DE => self.registers.get_de(),
            Register16bit::HL => self.registers.get_hl(),
        }
    }
    #[inline]
    fn load_16bit_value_into_register(&mut self, register16bit: Register16bit, value: u16) {
        match register16bit {
            Register16bit::BC => { self.registers.write_bc(value) }
            Register16bit::DE => { self.registers.write_de(value) }
            Register16bit::HL => { self.registers.write_hl(value) }
        };
        self.clock.m += 10;
    }
    #[inline]
    fn load_8bit_value_into_register(&mut self, register8bit: Register8bit, value: u8) {
        match register8bit {
            Register8bit::A => { self.registers.a = value }
            Register8bit::B => { self.registers.b = value }
            Register8bit::C => { self.registers.c = value }
            Register8bit::D => { self.registers.d = value }
            Register8bit::E => { self.registers.e = value }
            Register8bit::H => { self.registers.h = value }
            Register8bit::L => { self.registers.l = value }
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
        registers.flag(Z, v == 0).flag(N, false).flag(H, (v & 0x0F) + 1 > 0x0F);
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
            Register16bit::BC => { self.registers.write_bc(self.registers.get_bc().wrapping_add(1)) }
            Register16bit::DE => { self.registers.write_de(self.registers.get_de().wrapping_add(1)) }
            Register16bit::HL => { self.registers.write_hl(self.registers.get_hl().wrapping_add(1)) }
        };
        self.clock.m += 8;
    }
    #[inline]
    fn dec_8bit_register(&mut self, register8bit: Register8bit) {
        let register = self.get_register_refm_from_register_8bit(register8bit);
        *register = register.wrapping_sub(1);
        let temp_value = *register;
        self.registers.flag(Z, temp_value == 0).flag(N, true).flag(H, (temp_value & 0x0F) == 0);
    }
    #[inline]
    fn dec_16bit_register(&mut self, register16bit: Register16bit) {
        match register16bit {
            Register16bit::BC => { self.registers.write_bc(self.registers.get_bc().wrapping_sub(1)) }
            Register16bit::DE => { self.registers.write_de(self.registers.get_de().wrapping_sub(1)) }
            Register16bit::HL => { self.registers.write_hl(self.registers.get_hl().wrapping_sub(1)) }
        };
    }
}
