use crate::helper::{combine_u8_to_u16, split_u16};
use crate::memory::MEM_SIZE;

pub struct Registers{
    pub a:u8,
    pub b:u8,
    pub c:u8,
    pub d:u8,
    pub e:u8,
    f:u8,
    pub h:u8,
    pub l:u8,
    pub pc:u16,
    pub sp:u16
}
pub enum Flags{
    Z=0b10000000,
    N=0b01000000,
    H=0b00100000,
    C=0b00010000,
}
impl Default for Registers {
    fn default() -> Self {
        Self{
            a: 0,
            b: 0,
            c: 0,
            d: 0,
            e: 0,
            h: 0,
            l: 0,
            pc: 0,
            f:0,
            sp: (MEM_SIZE - 0xE) as u16,
        }
    }
}
impl Registers {

    #[inline]
    fn write_u16_into_two_u8(v:u16,f: &mut u8,s: &mut u8){
        let (first,second)=split_u16(v);
        *f=first;
        *s=second;
    }
    pub fn get_bc(&self)->u16{
        combine_u8_to_u16(self.b,self.c)
    }
    pub fn get_de(&self)->u16{
        combine_u8_to_u16(self.d,self.e)
    }
    pub fn get_hl(&self)->u16{
        combine_u8_to_u16(self.h,self.l)
    }
    pub fn get_af(&self)->u16{combine_u8_to_u16(self.a,self.f)}
    pub fn write_bc(&mut self,value:u16)->u16{
        Self::write_u16_into_two_u8(value, &mut self.b, &mut self.c);
        value
    }
    pub fn write_de(&mut self,value:u16)->u16{
        Self::write_u16_into_two_u8(value, &mut self.d, &mut self.e);
        value
    }
    pub fn write_af(&mut self,value:u16)->u16{
        Self::write_u16_into_two_u8(value, &mut self.a, &mut self.f);
        value
    }
    pub fn write_hl(&mut self,value:u16)->u16{
        Self::write_u16_into_two_u8(value, &mut self.h, &mut self.l);
        value
    }
    pub fn flag(&mut self,flag:Flags,set:bool)->&mut Self{
        let mask = flag as u8;
        if set { self.f|=mask; } else { self.f&=!mask; }
        self.f&=0xF0;
        self
    }
    pub fn get_flag(&self,flags: Flags)->bool{
        self.f&(flags as u8)>0
    }
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_registers() {
        let registers = Registers::default();
        assert_eq!(registers.a, 0);
        assert_eq!(registers.b, 0);
        assert_eq!(registers.c, 0);
        assert_eq!(registers.d, 0);
        assert_eq!(registers.e, 0);
        assert_eq!(registers.f, 0);
        assert_eq!(registers.h, 0);
        assert_eq!(registers.l, 0);
        assert_eq!(registers.pc, 0);
        assert_eq!(registers.sp, (MEM_SIZE as u16)-0xe);
    }

    #[test]
    fn test_get_bc() {
        let registers = Registers {
            b: 0x12,
            c: 0x34,
            ..Registers::default()
        };
        assert_eq!(registers.get_bc(), 0x1234);
    }

    #[test]
    fn test_get_de() {
        let registers = Registers {
            d: 0x56,
            e: 0x78,
            ..Registers::default()
        };
        assert_eq!(registers.get_de(), 0x5678);
    }

    #[test]
    fn test_get_hl() {
        let registers = Registers {
            h: 0x9A,
            l: 0xBC,
            ..Registers::default()
        };
        assert_eq!(registers.get_hl(), 0x9ABC);
    }

    #[test]
    fn test_write_bc() {
        let mut registers = Registers::default();
        registers.write_bc(0x1234);
        assert_eq!(registers.b, 0x12);
        assert_eq!(registers.c, 0x34);
    }

    #[test]
    fn test_write_de() {
        let mut registers = Registers::default();
        registers.write_de(0x5678);
        assert_eq!(registers.d, 0x56);
        assert_eq!(registers.e, 0x78);
    }

    #[test]
    fn test_write_hl() {
        let mut registers = Registers::default();
        registers.write_hl(0x9ABC);
        assert_eq!(registers.h, 0x9A);
        assert_eq!(registers.l, 0xBC);
    }

    #[test]
    fn test_flag_set() {
        let mut registers = Registers::default();
        registers.flag(Flags::Z, true);
        assert_eq!(registers.get_flag(Flags::Z), true);
        assert_eq!(registers.f, Flags::Z as u8);

        registers.flag(Flags::N, true);
        assert_eq!(registers.get_flag(Flags::N), true);
        assert_eq!(registers.f, ((Flags::Z as u8) | (Flags::N as u8)) as u8);
    }

    #[test]
    fn test_flag_clear() {
        let mut registers = Registers {
            f: Flags::Z as u8 | Flags::N as u8,
            ..Registers::default()
        };
        registers.flag(Flags::Z, false);
        assert_eq!(registers.get_flag(Flags::Z), false);
        assert_eq!(registers.get_flag(Flags::N), true);
        assert_eq!(registers.f, Flags::N as u8);
    }

    #[test]
    fn test_get_flag() {
        let mut registers = Registers::default();
        registers.flag(Flags::H, true);
        assert_eq!(registers.get_flag(Flags::H), true);

        registers.flag(Flags::C, true);
        assert_eq!(registers.get_flag(Flags::C), true);
    }

    #[test]
    fn test_write_af() {
        let mut registers = Registers::default();
        let value = 0x1234;
        registers.write_af(value);
        assert_eq!(registers.a, 0x12);  // Assuming `a` is the high byte
        assert_eq!(registers.f, 0x34);  // Assuming `f` is the low byte
    }

    #[test]
    fn test_get_af() {
        let registers = Registers {
            a: 0x12,
            f: 0x34,
            ..Registers::default()
        };
        assert_eq!(registers.get_af(), 0x1234);
    }
}
