use crate::cpu::Register16bit::{BC, DE, HL, SP};
use crate::cpu::Register8bit::{A, B, D, E, HlDirectMemory, L};
use crate::memory::Memory;
use crate::register::Flags::{C, H, N, Z};
use crate::register::Registers;

struct Clock {
    m: u128,
    t: u128,
}
struct Cpu {
    ime : bool,
    clock: Clock,
    halted:bool,
    memory: Memory,
    registers: Registers,
}
#[derive(Copy, Clone)]
enum Register8bit {
    A,
    B,
    C,
    D,
    E,
    H,
    L,
    HlDirectMemory,
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
            BC => self.registers.get_bc(),
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
        let reg = self.get_register_refm_from_register_8bit(register8bit);
        *reg = value;
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
            Register8bit::HlDirectMemory => self.memory.get_refm_to_byte(self.registers.get_hl()),
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
        self.clock.m += 16;
    }
    fn pop(&mut self) -> u16 {
        let stack_data = self.memory.read_word(self.registers.sp);
        self.registers.sp = self.registers.sp.wrapping_add(2);
        stack_data
    }
    fn pop_register16bit(&mut self, register16bit: Register16bit) {
        let stack_data = self.pop();
        match register16bit {
            Register16bit::BC => self.registers.write_bc(stack_data),
            Register16bit::DE => self.registers.write_de(stack_data),
            Register16bit::HL => self.registers.write_hl(stack_data),
            Register16bit::SP => {
                panic!("cant pop onto sp")
            }
        };
        self.clock.m += 12;
    }

    fn pop_af(&mut self) {
        let v = self.pop();
        self.registers.write_af(v);
        self.clock.m += 12;
    }
    fn call(&mut self, label: u16, cond: bool) {
        if cond {
            self.push(self.registers.pc);
            self.registers.pc = label;
            self.clock.m += 1
        } else {
            self.clock.m += 10
        }
    }
    //for some fucking reason pure ret does 10 cycles instead of 11 like the others
    //so there ia now a new function named pure_ret
    #[inline(always)]
    fn pure_ret(&mut self) {
        self.ret(true);
        self.clock.m -= 1;
    }
    fn ret(&mut self, cond: bool) {
        if cond {
            self.registers.pc = self.pop();
            self.clock.m += 11;
        } else {
            self.clock.m += 5;
        }
    }

    /// # Description
    ///  implements the RST Z80 command. pushes pc onto the stack and replaces its value
    ///  with `new_pc`
    /// # Arguments
    ///
    /// * `new_pc`: can be `0x00 , 0x10, 0x20, 0x30` *only*. (idk why but that's the way it is in the chip)
    ///
    /// returns: ()
    ///
    fn rst(&mut self, new_pc: u16) {
        self.push(self.registers.pc.wrapping_add(1));
        self.registers.pc = new_pc;
        self.clock.m += 1;
    }
    #[inline(always)]
    fn set_shift_flags(&mut self, result: u8, carry_flag: bool) {
        self.registers
            .flag(C, carry_flag)
            .flag(Z, result == 0)
            .flag(H, false)
            .flag(N, false);
    }
    #[inline(always)]
    fn shift_register8bit(
        &mut self,
        register8bit: Register8bit,
        c_version: bool,
        flag_check_number: u8,
        op: fn(u8) -> u8,
    ) {
        let carry_flag = self.registers.get_flag(C);
        let reg = self.get_register_refm_from_register_8bit(register8bit);
        let carry = *reg & flag_check_number == flag_check_number;
        let res = op(*reg)
            | if (if c_version { carry } else { carry_flag }) {
                1
            } else {
                0
            };
        *reg = res;
        self.set_shift_flags(res, carry);
        self.clock.m += if let Register8bit::A = register8bit {
            4
        } else {
            8
        };
    }
    fn shift_left(&mut self, register8bit: Register8bit) {
        self.shift_register8bit(register8bit, false, 0x80, |n| n << 1);
    }
    fn shift_left_c(&mut self, register8bit: Register8bit) {
        self.shift_register8bit(register8bit, true, 0x80, |n| n << 1);
    }
    fn shift_right_c(&mut self, register8bit: Register8bit) {
        self.shift_register8bit(register8bit, true, 0x01, |n| n >> 1);
    }
    fn shift_right(&mut self, register8bit: Register8bit) {
        self.shift_register8bit(register8bit, false, 0x01, |n| n >> 1);
    }
    fn swap(&mut self, register8bit: Register8bit) {
        let reg = self.get_register_refm_from_register_8bit(register8bit);
        let new_reg_value = (*reg >> 4) | (*reg << 4);
        *reg = new_reg_value;
        self.bit_op_set_flags(new_reg_value);
        self.clock.m += 2;
    }
    fn bit_op_set_flags(&mut self, result: u8) {
        self.registers
            .flag(Z, result == 0)
            .flag(H, false)
            .flag(C, false)
            .flag(N, false);
    }
    fn bit_op_register_8bit(&mut self, v: u8, op: fn(u8, u8) -> u8) {
        self.registers.a = op(self.registers.a, v);
        self.bit_op_set_flags(self.registers.a);
        self.clock.m += 2;
    }
    fn or(&mut self, v: u8) {
        self.bit_op_register_8bit(v, |i, i1| i | i1)
    }
    fn and(&mut self, v: u8) {
        self.bit_op_register_8bit(v, |i, i1| i & i1)
    }
    fn xor(&mut self, v: u8) {
        self.bit_op_register_8bit(v, |i, i1| i ^ i1)
    }
    fn cmp(&mut self, v: u8) {
        let a = self.registers.a;
        self.sub_8bit_value(a, false);
        self.registers.a = a;
    }
    fn jump(&mut self, loc: u16, cond: bool) {
        if cond {
            self.registers.pc = loc;
        }
        self.clock.m += 10;
    }
    fn jr(&mut self, loc: i16, cond: bool) {
        if cond {
            self.registers.pc += if loc.is_negative() {
                self.registers.pc.wrapping_sub(loc.abs() as u16)
            } else {
                self.registers.pc.wrapping_add(loc as u16)
            };
            self.clock.m += 12;
        } else {
            self.clock.m += 8;
        }
    }
    fn read_next(&mut self)->u8{
        let ret =self.memory.read_byte(self.registers.pc);
        self.registers.pc+=1;
        ret
    }
    fn read_next_word(&mut self)->u16{
        let ret = self.memory.read_word(self.registers.pc);
        self.registers.pc+=2;
        ret
    }
    fn load_direct_mem(&mut self,register16bit: Register16bit,value:u8){
        let reg = self.get_register_from_register16bit(register16bit);
        self.memory.write_byte(reg,value);
        self.clock.m+=7;
    }
    fn bit(&mut self,bit:u8,register8bit: Register8bit){
        let reg = self.get_register_refm_from_register_8bit(register8bit);
        let res = *reg & (1<<(bit as u32)) == 0;
        self.registers.flag(N,false).flag(H,true).flag(Z,res);
        self.clock.m+=8;
    }
    fn res(&mut self,bit:u8,register8bit: Register8bit){
        let reg = self.get_register_refm_from_register_8bit(register8bit);
        let reg_value = *reg;
        *reg = reg_value & (1 << bit);
        self.clock.m+=8;
    }
    fn set(&mut self,bit:u8,register8bit: Register8bit){
        let reg = self.get_register_refm_from_register_8bit(register8bit);
        let reg_value = *reg;
        *reg = reg_value | (1 << bit);
        self.clock.m+=8;
    }
    fn daa(&mut self){
        let mut a =self.registers.a;
        let mut adjust = if self.registers.get_flag(C) {0x60} else { 0x00 };
        if self.registers.get_flag(H) {adjust|=0x06;};
        if !self.registers.get_flag(N) {
            if a & 0x0F >0x09 {adjust|=0x06;};
            if a > 0x99 {adjust|=0x60};
            a = a.wrapping_add(adjust);
        } else {
            a = a.wrapping_sub(adjust);
        }
        self.registers.flag(C,adjust>=0x60).flag(H,false).flag(Z,a==0);
        self.registers.a=a;
    }
    fn sla(&mut self,register8bit: Register8bit){
        let reg = self.get_register_refm_from_register_8bit(register8bit);
        let v = *reg;
        let carry_flag = v & 0x80 == 0x80;
        let res = v <<1;
        *reg = res;
        self.registers.flag(Z,v == 0).flag(H,false).flag(N,false).flag(C,carry_flag);
        self.clock.m+=8;
    }
    fn sra(&mut self,register8bit: Register8bit){
        let reg = self.get_register_refm_from_register_8bit(register8bit);
        let v = *reg;
        let carry_flag = v & 0x01 == 0x01;
        let res = (v >> 1) | (v & 0x80);
        *reg = res;
        self.registers.flag(Z,v == 0).flag(H,false).flag(N,false).flag(C,carry_flag);
        self.clock.m+=8;
    }
    fn srl(&mut self,register8bit: Register8bit){
        let reg = self.get_register_refm_from_register_8bit(register8bit);
        let v = *reg;
        let carry_flag = v & 0x01 == 0x01;
        let res = v >> 1;
        *reg = res;
        self.registers.flag(Z,v == 0).flag(H,false).flag(N,false).flag(C,carry_flag);
        self.clock.m+=8;
    }
    fn exec(&mut self){
        let op_code = self.read_next();
        match op_code {
            0x00=>self.clock.m+=1,
            0x01=>{let v = self.read_next_word();self.load_16bit_value_into_register(Register16bit::BC,v)}
            0x02=>{self.load_direct_mem(Register16bit::BC,self.registers.a)}
            0x03=>self.inc_16bit_register(BC),
            0x04=>self.inc_8bit_register(B),
            0x05=>self.dec_8bit_register(B),
            0x06 => {let v = self.read_next();self.load_8bit_value_into_register(B,v)}
            0x07 => self.shift_left_c(A),
            0x08 => {let v = self.read_next_word(); self.memory.write_word(v,self.registers.sp);self.clock.m+=20}
            0x09 => self.add_16bit_value(self.registers.get_bc()),
            0x0A => self.load_8bit_value_into_register(A,self.memory.read_byte(self.registers.get_bc())),
            0x0B =>self.dec_16bit_register(BC),
            0x0C => self.inc_8bit_register(Register8bit::C),
            0x0D => self.dec_8bit_register(Register8bit::C),
            0x0E => {let v = self.read_next();self.load_8bit_value_into_register(Register8bit::C,v)}
            0x0F=> self.shift_right_c(A),
            0x10=>self.clock.m+=4,
            0x11 => {let v =self.read_next_word();self.load_16bit_value_into_register(Register16bit::DE,v)}
            0x12 => self.load_direct_mem(DE,self.registers.a),
            0x13 => self.inc_16bit_register(DE),
            0x14 => self.inc_8bit_register(D),
            0x15 => self.dec_8bit_register(D),
            0x16 => {let v = self.read_next();self.load_8bit_value_into_register(D,v)}
            0x17 => self.shift_left(A),
            0x18 => {let v = self.read_next() as i8 as i16;self.jr(v,true)}
            0x19 => self.add_16bit_value(self.registers.get_de()),
            0x1A => self.load_8bit_value_into_register(A,self.memory.read_byte(self.registers.get_de())),
            0x1B => self.dec_16bit_register(DE),
            0x1C => self.inc_8bit_register(Register8bit::E),
            0x1D => self.dec_8bit_register(Register8bit::E),
            0x1E => {let v =self.read_next(); self.load_8bit_value_into_register(E,v)}
            0x1F => self.shift_right(A),
            0x20 => {let v = self.read_next() as i8 as i16; self.jr(v,!self.registers.get_flag(Z))}
            0x21 => {let v = self.read_next_word(); self.load_16bit_value_into_register(HL,v)}
            0x22 => {self.load_direct_mem(HL,self.registers.a);self.inc_16bit_register(HL);self.clock.m-=7}
            0x23 => self.inc_16bit_register(HL),
            0x24 => self.inc_8bit_register(Register8bit::H),
            0x25 => self.dec_8bit_register(Register8bit::H),
            0x26 => {let v = self.read_next();self.load_8bit_value_into_register(Register8bit::H,v)}
            0x27 => self.daa(),
            0x28 => {let v = self.read_next()as i8 as i16;self.jr(v,self.registers.get_flag(Z))}
            0x29 =>self.add_16bit_value(self.registers.get_hl()),
            0x2A => {self.load_8bit_value_into_register(A,self.memory.read_byte(self.registers.get_hl()));self.inc_16bit_register(HL);self.clock.m-=7}
            0x2B => self.dec_16bit_register(HL),
            0x2C=>self.inc_8bit_register(L),
            0x2D => self.dec_8bit_register(L),
            0x2E => {let v =self.read_next(); self.load_8bit_value_into_register(L,v)}
            0x2F => {self.registers.a= !self.registers.a;self.registers.flag(H,true).flag(N,true);}
            0x30 => {let v = self.read_next() as i8 as i16; self.jr(v, !self.registers.get_flag(C)) }
            0x31 => {let v =self.read_next_word(); self.load_16bit_value_into_register(SP,v)}
            0x32 => {self.load_direct_mem(HL,self.registers.a);self.dec_16bit_register(HL);self.clock.m-=7}
            0x33 => self.inc_16bit_register(SP),
            0x34=> self.inc_8bit_register(HlDirectMemory),
            0x35=>self.dec_8bit_register(HlDirectMemory),
            0x36 => {let v =self.read_next();self.load_direct_mem(HL,v);}
            0x37 => {self.registers.flag(C,true).flag(N,false).flag(H,false);}
            0x38 => {let v = self.read_next() as i8 as i16; self.jr(v,self.registers.get_flag(C))}
            0x39 => self.add_16bit_value(self.registers.sp),
            0x3A => {self.load_8bit_value_into_register(A,self.memory.read_byte(self.registers.get_hl()));self.dec_16bit_register(HL);self.clock.m-=7}
            0x3B => self.dec_16bit_register(SP),
            0x3C => self.inc_8bit_register(A),
            0x3D => self.dec_8bit_register(A),
            0x3E => {let v = self.read_next(); self.load_8bit_value_into_register(A,v)}
            0x3F => {let v = !self.registers.get_flag(C);self.registers.flag(C,v).flag(H,false).flag(N,false);}
            // generated load commands
            0x40 => self.load_8bit_value_into_register(Register8bit::B,self.registers.b),
            0x41 => self.load_8bit_value_into_register(Register8bit::B,self.registers.c),
            0x42 => self.load_8bit_value_into_register(Register8bit::B,self.registers.d),
            0x43 => self.load_8bit_value_into_register(Register8bit::B,self.registers.e),
            0x44 => self.load_8bit_value_into_register(Register8bit::B,self.registers.h),
            0x45 => self.load_8bit_value_into_register(Register8bit::B,self.registers.l),
            0x46 => self.load_8bit_value_into_register(Register8bit::B,self.memory.read_byte(self.registers.get_hl())),
            0x47 => self.load_8bit_value_into_register(Register8bit::B,self.registers.a),
            0x48 => self.load_8bit_value_into_register(Register8bit::C,self.registers.b),
            0x49 => self.load_8bit_value_into_register(Register8bit::C,self.registers.c),
            0x4a => self.load_8bit_value_into_register(Register8bit::C,self.registers.d),
            0x4b => self.load_8bit_value_into_register(Register8bit::C,self.registers.e),
            0x4c => self.load_8bit_value_into_register(Register8bit::C,self.registers.h),
            0x4d => self.load_8bit_value_into_register(Register8bit::C,self.registers.l),
            0x4e => self.load_8bit_value_into_register(Register8bit::C,self.memory.read_byte(self.registers.get_hl())),
            0x4f => self.load_8bit_value_into_register(Register8bit::C,self.registers.a),
            0x50 => self.load_8bit_value_into_register(Register8bit::D,self.registers.b),
            0x51 => self.load_8bit_value_into_register(Register8bit::D,self.registers.c),
            0x52 => self.load_8bit_value_into_register(Register8bit::D,self.registers.d),
            0x53 => self.load_8bit_value_into_register(Register8bit::D,self.registers.e),
            0x54 => self.load_8bit_value_into_register(Register8bit::D,self.registers.h),
            0x55 => self.load_8bit_value_into_register(Register8bit::D,self.registers.l),
            0x56 => self.load_8bit_value_into_register(Register8bit::D,self.memory.read_byte(self.registers.get_hl())),
            0x57 => self.load_8bit_value_into_register(Register8bit::D,self.registers.a),
            0x58 => self.load_8bit_value_into_register(Register8bit::E,self.registers.b),
            0x59 => self.load_8bit_value_into_register(Register8bit::E,self.registers.c),
            0x5a => self.load_8bit_value_into_register(Register8bit::E,self.registers.d),
            0x5b => self.load_8bit_value_into_register(Register8bit::E,self.registers.e),
            0x5c => self.load_8bit_value_into_register(Register8bit::E,self.registers.h),
            0x5d => self.load_8bit_value_into_register(Register8bit::E,self.registers.l),
            0x5e => self.load_8bit_value_into_register(Register8bit::E,self.memory.read_byte(self.registers.get_hl())),
            0x5f => self.load_8bit_value_into_register(Register8bit::E,self.registers.a),
            0x60 => self.load_8bit_value_into_register(Register8bit::H,self.registers.b),
            0x61 => self.load_8bit_value_into_register(Register8bit::H,self.registers.c),
            0x62 => self.load_8bit_value_into_register(Register8bit::H,self.registers.d),
            0x63 => self.load_8bit_value_into_register(Register8bit::H,self.registers.e),
            0x64 => self.load_8bit_value_into_register(Register8bit::H,self.registers.h),
            0x65 => self.load_8bit_value_into_register(Register8bit::H,self.registers.l),
            0x66 => self.load_8bit_value_into_register(Register8bit::H,self.memory.read_byte(self.registers.get_hl())),
            0x67 => self.load_8bit_value_into_register(Register8bit::H,self.registers.a),
            0x68 => self.load_8bit_value_into_register(Register8bit::L,self.registers.b),
            0x69 => self.load_8bit_value_into_register(Register8bit::L,self.registers.c),
            0x6a => self.load_8bit_value_into_register(Register8bit::L,self.registers.d),
            0x6b => self.load_8bit_value_into_register(Register8bit::L,self.registers.e),
            0x6c => self.load_8bit_value_into_register(Register8bit::L,self.registers.h),
            0x6d => self.load_8bit_value_into_register(Register8bit::L,self.registers.l),
            0x6e => self.load_8bit_value_into_register(Register8bit::L,self.memory.read_byte(self.registers.get_hl())),
            0x6f => self.load_8bit_value_into_register(Register8bit::L,self.registers.a),
            0x70 => self.load_8bit_value_into_register(Register8bit::HlDirectMemory,self.registers.b),
            0x71 => self.load_8bit_value_into_register(Register8bit::HlDirectMemory,self.registers.c),
            0x72 => self.load_8bit_value_into_register(Register8bit::HlDirectMemory,self.registers.d),
            0x73 => self.load_8bit_value_into_register(Register8bit::HlDirectMemory,self.registers.e),
            0x74 => self.load_8bit_value_into_register(Register8bit::HlDirectMemory,self.registers.h),
            0x75 => self.load_8bit_value_into_register(Register8bit::HlDirectMemory,self.registers.l),
            0x76 => {self.halted=true;self.clock.m+=4}
            0x77 => self.load_8bit_value_into_register(Register8bit::HlDirectMemory,self.registers.a),
            0x78 => self.load_8bit_value_into_register(Register8bit::A,self.registers.b),
            0x79 => self.load_8bit_value_into_register(Register8bit::A,self.registers.c),
            0x7a => self.load_8bit_value_into_register(Register8bit::A,self.registers.d),
            0x7b => self.load_8bit_value_into_register(Register8bit::A,self.registers.e),
            0x7c => self.load_8bit_value_into_register(Register8bit::A,self.registers.h),
            0x7d => self.load_8bit_value_into_register(Register8bit::A,self.registers.l),
            0x7e => self.load_8bit_value_into_register(Register8bit::A,self.memory.read_byte(self.registers.get_hl())),
            0x7f => self.load_8bit_value_into_register(Register8bit::A,self.registers.a),
            //genereated alu functions
            0x80 => self.add_8bit_value(self.registers.b,false),
            0x81 => self.add_8bit_value(self.registers.c,false),
            0x82 => self.add_8bit_value(self.registers.d,false),
            0x83 => self.add_8bit_value(self.registers.e,false),
            0x84 => self.add_8bit_value(self.registers.h,false),
            0x85 => self.add_8bit_value(self.registers.l,false),
            0x86 => self.add_8bit_value(self.memory.read_byte(self.registers.get_hl()),false),
            0x87 => self.add_8bit_value(self.registers.a,false),
            0x88 => self.add_8bit_value(self.registers.b,true),
            0x89 => self.add_8bit_value(self.registers.c,true),
            0x8a => self.add_8bit_value(self.registers.d,true),
            0x8b => self.add_8bit_value(self.registers.e,true),
            0x8c => self.add_8bit_value(self.registers.h,true),
            0x8d => self.add_8bit_value(self.registers.l,true),
            0x8e => self.add_8bit_value(self.memory.read_byte(self.registers.get_hl()),true),
            0x8f => self.add_8bit_value(self.registers.a,true),
            0x90 => self.sub_8bit_value(self.registers.b,false),
            0x91 => self.sub_8bit_value(self.registers.c,false),
            0x92 => self.sub_8bit_value(self.registers.d,false),
            0x93 => self.sub_8bit_value(self.registers.e,false),
            0x94 => self.sub_8bit_value(self.registers.h,false),
            0x95 => self.sub_8bit_value(self.registers.l,false),
            0x96 => self.sub_8bit_value(self.memory.read_byte(self.registers.get_hl()),false),
            0x97 => self.sub_8bit_value(self.registers.a,false),
            0x98 => self.sub_8bit_value(self.registers.b,true),
            0x99 => self.sub_8bit_value(self.registers.c,true),
            0x9a => self.sub_8bit_value(self.registers.d,true),
            0x9b => self.sub_8bit_value(self.registers.e,true),
            0x9c => self.sub_8bit_value(self.registers.h,true),
            0x9d => self.sub_8bit_value(self.registers.l,true),
            0x9e => self.sub_8bit_value(self.memory.read_byte(self.registers.get_hl()),true),
            0x9f => self.sub_8bit_value(self.registers.a,true),
            0xa0 => self.and(self.registers.b),
            0xa1 => self.and(self.registers.c),
            0xa2 => self.and(self.registers.d),
            0xa3 => self.and(self.registers.e),
            0xa4 => self.and(self.registers.h),
            0xa5 => self.and(self.registers.l),
            0xa6 => self.and(self.memory.read_byte(self.registers.get_hl())),
            0xa7 => self.and(self.registers.a),
            0xa8 => self.xor(self.registers.b),
            0xa9 => self.xor(self.registers.c),
            0xaa => self.xor(self.registers.d),
            0xab => self.xor(self.registers.e),
            0xac => self.xor(self.registers.h),
            0xad => self.xor(self.registers.l),
            0xae => self.xor(self.memory.read_byte(self.registers.get_hl())),
            0xaf => self.xor(self.registers.a),
            0xb0 => self.or(self.registers.b),
            0xb1 => self.or(self.registers.c),
            0xb2 => self.or(self.registers.d),
            0xb3 => self.or(self.registers.e),
            0xb4 => self.or(self.registers.h),
            0xb5 => self.or(self.registers.l),
            0xb6 => self.or(self.memory.read_byte(self.registers.get_hl())),
            0xb7 => self.or(self.registers.a),
            0xb8 => self.cmp(self.registers.b),
            0xb9 => self.cmp(self.registers.c),
            0xba => self.cmp(self.registers.d),
            0xbb => self.cmp(self.registers.e),
            0xbc => self.cmp(self.registers.h),
            0xbd => self.cmp(self.registers.l),
            0xbe => self.cmp(self.memory.read_byte(self.registers.get_hl())),
            0xbf => self.cmp(self.registers.a),
            0xC0 => self.ret(!self.registers.get_flag(Z)),
            0xC1 => self.pop_register16bit(BC),
            0xC2 => {let v = self.read_next_word(); self.jump(v, !self.registers.get_flag(Z))}
            0xC3 => {let v = self.read_next_word(); self.jump(v, true)}
            0xC4 => {let v = self.read_next_word(); self.call(v, !self.registers.get_flag(Z))}
            0xC5 => self.push(self.registers.get_bc()),
            0xC6 => {let v = self.read_next(); self.add_8bit_value(v,false)}
            0xC7 => self.rst(0x00),
            0xC8 => self.ret(self.registers.get_flag(Z)),
            0xC9 => self.ret(true),
            0xCA => {let v = self.read_next_word(); self.jump(v, self.registers.get_flag(Z))}
            0xCB => todo!("impl cb calls"),
            0xCC => {let v = self.read_next_word(); self.call(v, self.registers.get_flag(Z))}
            0xCD => {let v = self.read_next_word(); self.call(v, true)}
            0xCE => {let v = self.read_next(); self.add_8bit_value(v,true)}
            0xCF => self.rst(0x08),
            0xD0 => self.ret(!self.registers.get_flag(C)),
            0xD1 => self.pop_register16bit(DE),
            0xD2 => {let v = self.read_next_word(); self.jump(v, !self.registers.get_flag(C))}
            0xD4 => {let v = self.read_next_word(); self.call(v, !self.registers.get_flag(C))}
            0xD5 => self.push(self.registers.get_bc()),
            0xD6 => {let v = self.read_next(); self.sub_8bit_value(v,false)}
            0xD7 => self.rst(0x10),
            0xD8 => self.ret(self.registers.get_flag(C)),
            0xD9 => {self.ret(true);self.ime=true;}
            0xDA => {let v = self.read_next_word(); self.jump(v, self.registers.get_flag(C))}
            0xDC => {let v = self.read_next_word(); self.call(v, self.registers.get_flag(C))}
            0xDE => {let v = self.read_next(); self.sub_8bit_value(v,true)}
            0xDF => self.rst(0x18),
            0xE0 => {let v = self.read_next(); self.memory.write_byte(0xFF00 + (v as u16), self.registers.a)}
            0xE1 => self.pop_register16bit(HL),
            0xE2 => self.memory.write_byte(0xFF00 + (self.registers.c as u16),self.registers.a),
            0xE5 => self.push(self.registers.get_hl()),
            0xE6 => {let v = self.read_next();self.and(v)}
            0xE7 => self.rst(0x20),
            0xE8 => {let v = self.read_next() as i8; if v.is_negative(){self.sub_8bit_value(v.abs() as u8, false)} else {self.add_8bit_value(v as u8,false)}}
            0xE9 => self.jump(self.registers.get_hl(),true),
            0xEA => {let v =self.read_next_word(); self.memory.write_byte(v,self.registers.a)}
            0xEE => {let v = self.read_next(); self.xor(v)}
            0xEF => self.rst(0x28),
            0xF0 => {let v = self.read_next(); self.load_8bit_value_into_register(A,self.memory.read_byte(0xFF00+(v as u16)))}
            0xF1 => self.pop_af(),
            0xF2 => self.load_8bit_value_into_register(A,self.memory.read_byte(0xFF00+(self.registers.c as u16))),
            0xF3=>{self.ime=false;self.clock.m+=4}
            0xF5 => self.push(self.registers.get_af()),
            0xF6 => {let v = self.read_next(); self.or(v)}
            0xF7 => self.rst(0x30),
            0xF8 => {let v = self.read_next() as i8 as i16; self.load_16bit_value_into_register(HL,((self.registers.sp as i16) + v) as u16)}
            0xF9 => self.load_16bit_value_into_register(SP,self.registers.get_hl()),
            0xFA => {let v = self.read_next_word(); self.load_8bit_value_into_register(A,self.memory.read_byte(v))}
            0xFB =>{self.ime=true;self.clock.m+=4}
            0xFE => {let v = self.read_next(); self.cmp(v)}
            0xFF => self.rst(0x38),
            _=>panic!("op code not implemented: {}",op_code)
        }
    }
    fn call_cb(&mut self){
        let code =self.read_next();
        match code{
            0x0 => self.shift_left_c(Register8bit::B),
            0x1 => self.shift_left_c(Register8bit::C),
            0x2 => self.shift_left_c(Register8bit::D),
            0x3 => self.shift_left_c(Register8bit::E),
            0x4 => self.shift_left_c(Register8bit::H),
            0x5 => self.shift_left_c(Register8bit::L),
            0x6 => self.shift_left_c(Register8bit::HlDirectMemory),
            0x7 => self.shift_left_c(Register8bit::A),
            0x8 => self.shift_right_c(Register8bit::B),
            0x9 => self.shift_right_c(Register8bit::C),
            0xa => self.shift_right_c(Register8bit::D),
            0xb => self.shift_right_c(Register8bit::E),
            0xc => self.shift_right_c(Register8bit::H),
            0xd => self.shift_right_c(Register8bit::L),
            0xe => self.shift_right_c(Register8bit::HlDirectMemory),
            0xf => self.shift_right_c(Register8bit::A),
            0x10 => self.shift_left(Register8bit::B),
            0x11 => self.shift_left(Register8bit::C),
            0x12 => self.shift_left(Register8bit::D),
            0x13 => self.shift_left(Register8bit::E),
            0x14 => self.shift_left(Register8bit::H),
            0x15 => self.shift_left(Register8bit::L),
            0x16 => self.shift_left(Register8bit::HlDirectMemory),
            0x17 => self.shift_left(Register8bit::A),
            0x18 => self.shift_right(Register8bit::B),
            0x19 => self.shift_right(Register8bit::C),
            0x1a => self.shift_right(Register8bit::D),
            0x1b => self.shift_right(Register8bit::E),
            0x1c => self.shift_right(Register8bit::H),
            0x1d => self.shift_right(Register8bit::L),
            0x1e => self.shift_right(Register8bit::HlDirectMemory),
            0x1f => self.shift_right(Register8bit::A),
            0x20 => self.sla(Register8bit::B),
            0x21 => self.sla(Register8bit::C),
            0x22 => self.sla(Register8bit::D),
            0x23 => self.sla(Register8bit::E),
            0x24 => self.sla(Register8bit::H),
            0x25 => self.sla(Register8bit::L),
            0x26 => self.sla(Register8bit::HlDirectMemory),
            0x27 => self.sla(Register8bit::A),
            0x28 => self.sra(Register8bit::B),
            0x29 => self.sra(Register8bit::C),
            0x2a => self.sra(Register8bit::D),
            0x2b => self.sra(Register8bit::E),
            0x2c => self.sra(Register8bit::H),
            0x2d => self.sra(Register8bit::L),
            0x2e => self.sra(Register8bit::HlDirectMemory),
            0x2f => self.sra(Register8bit::A),
            0x30 => self.swap(Register8bit::B),
            0x31 => self.swap(Register8bit::C),
            0x32 => self.swap(Register8bit::D),
            0x33 => self.swap(Register8bit::E),
            0x34 => self.swap(Register8bit::H),
            0x35 => self.swap(Register8bit::L),
            0x36 => self.swap(Register8bit::HlDirectMemory),
            0x37 => self.swap(Register8bit::A),
            0x38 => self.srl(Register8bit::B),
            0x39 => self.srl(Register8bit::C),
            0x3a => self.srl(Register8bit::D),
            0x3b => self.srl(Register8bit::E),
            0x3c => self.srl(Register8bit::H),
            0x3d => self.srl(Register8bit::L),
            0x3e => self.srl(Register8bit::HlDirectMemory),
            0x3f => self.srl(Register8bit::A),
            0x40 => self.bit(0,Register8bit::B),
            0x41 => self.bit(0,Register8bit::C),
            0x42 => self.bit(0,Register8bit::D),
            0x43 => self.bit(0,Register8bit::E),
            0x44 => self.bit(0,Register8bit::H),
            0x45 => self.bit(0,Register8bit::L),
            0x46 => self.bit(0,Register8bit::HlDirectMemory),
            0x47 => self.bit(0,Register8bit::A),
            0x48 => self.bit(1,Register8bit::B),
            0x49 => self.bit(1,Register8bit::C),
            0x4a => self.bit(1,Register8bit::D),
            0x4b => self.bit(1,Register8bit::E),
            0x4c => self.bit(1,Register8bit::H),
            0x4d => self.bit(1,Register8bit::L),
            0x4e => self.bit(1,Register8bit::HlDirectMemory),
            0x4f => self.bit(1,Register8bit::A),
            0x50 => self.bit(2,Register8bit::B),
            0x51 => self.bit(2,Register8bit::C),
            0x52 => self.bit(2,Register8bit::D),
            0x53 => self.bit(2,Register8bit::E),
            0x54 => self.bit(2,Register8bit::H),
            0x55 => self.bit(2,Register8bit::L),
            0x56 => self.bit(2,Register8bit::HlDirectMemory),
            0x57 => self.bit(2,Register8bit::A),
            0x58 => self.bit(3,Register8bit::B),
            0x59 => self.bit(3,Register8bit::C),
            0x5a => self.bit(3,Register8bit::D),
            0x5b => self.bit(3,Register8bit::E),
            0x5c => self.bit(3,Register8bit::H),
            0x5d => self.bit(3,Register8bit::L),
            0x5e => self.bit(3,Register8bit::HlDirectMemory),
            0x5f => self.bit(3,Register8bit::A),
            0x60 => self.bit(4,Register8bit::B),
            0x61 => self.bit(4,Register8bit::C),
            0x62 => self.bit(4,Register8bit::D),
            0x63 => self.bit(4,Register8bit::E),
            0x64 => self.bit(4,Register8bit::H),
            0x65 => self.bit(4,Register8bit::L),
            0x66 => self.bit(4,Register8bit::HlDirectMemory),
            0x67 => self.bit(4,Register8bit::A),
            0x68 => self.bit(5,Register8bit::B),
            0x69 => self.bit(5,Register8bit::C),
            0x6a => self.bit(5,Register8bit::D),
            0x6b => self.bit(5,Register8bit::E),
            0x6c => self.bit(5,Register8bit::H),
            0x6d => self.bit(5,Register8bit::L),
            0x6e => self.bit(5,Register8bit::HlDirectMemory),
            0x6f => self.bit(5,Register8bit::A),
            0x70 => self.bit(6,Register8bit::B),
            0x71 => self.bit(6,Register8bit::C),
            0x72 => self.bit(6,Register8bit::D),
            0x73 => self.bit(6,Register8bit::E),
            0x74 => self.bit(6,Register8bit::H),
            0x75 => self.bit(6,Register8bit::L),
            0x76 => self.bit(6,Register8bit::HlDirectMemory),
            0x77 => self.bit(6,Register8bit::A),
            0x78 => self.bit(7,Register8bit::B),
            0x79 => self.bit(7,Register8bit::C),
            0x7a => self.bit(7,Register8bit::D),
            0x7b => self.bit(7,Register8bit::E),
            0x7c => self.bit(7,Register8bit::H),
            0x7d => self.bit(7,Register8bit::L),
            0x7e => self.bit(7,Register8bit::HlDirectMemory),
            0x7f => self.bit(7,Register8bit::A),
            0x80 => self.res(0,Register8bit::B),
            0x81 => self.res(0,Register8bit::C),
            0x82 => self.res(0,Register8bit::D),
            0x83 => self.res(0,Register8bit::E),
            0x84 => self.res(0,Register8bit::H),
            0x85 => self.res(0,Register8bit::L),
            0x86 => self.res(0,Register8bit::HlDirectMemory),
            0x87 => self.res(0,Register8bit::A),
            0x88 => self.res(1,Register8bit::B),
            0x89 => self.res(1,Register8bit::C),
            0x8a => self.res(1,Register8bit::D),
            0x8b => self.res(1,Register8bit::E),
            0x8c => self.res(1,Register8bit::H),
            0x8d => self.res(1,Register8bit::L),
            0x8e => self.res(1,Register8bit::HlDirectMemory),
            0x8f => self.res(1,Register8bit::A),
            0x90 => self.res(2,Register8bit::B),
            0x91 => self.res(2,Register8bit::C),
            0x92 => self.res(2,Register8bit::D),
            0x93 => self.res(2,Register8bit::E),
            0x94 => self.res(2,Register8bit::H),
            0x95 => self.res(2,Register8bit::L),
            0x96 => self.res(2,Register8bit::HlDirectMemory),
            0x97 => self.res(2,Register8bit::A),
            0x98 => self.res(3,Register8bit::B),
            0x99 => self.res(3,Register8bit::C),
            0x9a => self.res(3,Register8bit::D),
            0x9b => self.res(3,Register8bit::E),
            0x9c => self.res(3,Register8bit::H),
            0x9d => self.res(3,Register8bit::L),
            0x9e => self.res(3,Register8bit::HlDirectMemory),
            0x9f => self.res(3,Register8bit::A),
            0xa0 => self.res(4,Register8bit::B),
            0xa1 => self.res(4,Register8bit::C),
            0xa2 => self.res(4,Register8bit::D),
            0xa3 => self.res(4,Register8bit::E),
            0xa4 => self.res(4,Register8bit::H),
            0xa5 => self.res(4,Register8bit::L),
            0xa6 => self.res(4,Register8bit::HlDirectMemory),
            0xa7 => self.res(4,Register8bit::A),
            0xa8 => self.res(5,Register8bit::B),
            0xa9 => self.res(5,Register8bit::C),
            0xaa => self.res(5,Register8bit::D),
            0xab => self.res(5,Register8bit::E),
            0xac => self.res(5,Register8bit::H),
            0xad => self.res(5,Register8bit::L),
            0xae => self.res(5,Register8bit::HlDirectMemory),
            0xaf => self.res(5,Register8bit::A),
            0xb0 => self.res(6,Register8bit::B),
            0xb1 => self.res(6,Register8bit::C),
            0xb2 => self.res(6,Register8bit::D),
            0xb3 => self.res(6,Register8bit::E),
            0xb4 => self.res(6,Register8bit::H),
            0xb5 => self.res(6,Register8bit::L),
            0xb6 => self.res(6,Register8bit::HlDirectMemory),
            0xb7 => self.res(6,Register8bit::A),
            0xb8 => self.res(7,Register8bit::B),
            0xb9 => self.res(7,Register8bit::C),
            0xba => self.res(7,Register8bit::D),
            0xbb => self.res(7,Register8bit::E),
            0xbc => self.res(7,Register8bit::H),
            0xbd => self.res(7,Register8bit::L),
            0xbe => self.res(7,Register8bit::HlDirectMemory),
            0xbf => self.res(7,Register8bit::A),
            0xc0 => self.set(0,Register8bit::B),
            0xc1 => self.set(0,Register8bit::C),
            0xc2 => self.set(0,Register8bit::D),
            0xc3 => self.set(0,Register8bit::E),
            0xc4 => self.set(0,Register8bit::H),
            0xc5 => self.set(0,Register8bit::L),
            0xc6 => self.set(0,Register8bit::HlDirectMemory),
            0xc7 => self.set(0,Register8bit::A),
            0xc8 => self.set(1,Register8bit::B),
            0xc9 => self.set(1,Register8bit::C),
            0xca => self.set(1,Register8bit::D),
            0xcb => self.set(1,Register8bit::E),
            0xcc => self.set(1,Register8bit::H),
            0xcd => self.set(1,Register8bit::L),
            0xce => self.set(1,Register8bit::HlDirectMemory),
            0xcf => self.set(1,Register8bit::A),
            0xd0 => self.set(2,Register8bit::B),
            0xd1 => self.set(2,Register8bit::C),
            0xd2 => self.set(2,Register8bit::D),
            0xd3 => self.set(2,Register8bit::E),
            0xd4 => self.set(2,Register8bit::H),
            0xd5 => self.set(2,Register8bit::L),
            0xd6 => self.set(2,Register8bit::HlDirectMemory),
            0xd7 => self.set(2,Register8bit::A),
            0xd8 => self.set(3,Register8bit::B),
            0xd9 => self.set(3,Register8bit::C),
            0xda => self.set(3,Register8bit::D),
            0xdb => self.set(3,Register8bit::E),
            0xdc => self.set(3,Register8bit::H),
            0xdd => self.set(3,Register8bit::L),
            0xde => self.set(3,Register8bit::HlDirectMemory),
            0xdf => self.set(3,Register8bit::A),
            0xe0 => self.set(4,Register8bit::B),
            0xe1 => self.set(4,Register8bit::C),
            0xe2 => self.set(4,Register8bit::D),
            0xe3 => self.set(4,Register8bit::E),
            0xe4 => self.set(4,Register8bit::H),
            0xe5 => self.set(4,Register8bit::L),
            0xe6 => self.set(4,Register8bit::HlDirectMemory),
            0xe7 => self.set(4,Register8bit::A),
            0xe8 => self.set(5,Register8bit::B),
            0xe9 => self.set(5,Register8bit::C),
            0xea => self.set(5,Register8bit::D),
            0xeb => self.set(5,Register8bit::E),
            0xec => self.set(5,Register8bit::H),
            0xed => self.set(5,Register8bit::L),
            0xee => self.set(5,Register8bit::HlDirectMemory),
            0xef => self.set(5,Register8bit::A),
            0xf0 => self.set(6,Register8bit::B),
            0xf1 => self.set(6,Register8bit::C),
            0xf2 => self.set(6,Register8bit::D),
            0xf3 => self.set(6,Register8bit::E),
            0xf4 => self.set(6,Register8bit::H),
            0xf5 => self.set(6,Register8bit::L),
            0xf6 => self.set(6,Register8bit::HlDirectMemory),
            0xf7 => self.set(6,Register8bit::A),
            0xf8 => self.set(7,Register8bit::B),
            0xf9 => self.set(7,Register8bit::C),
            0xfa => self.set(7,Register8bit::D),
            0xfb => self.set(7,Register8bit::E),
            0xfc => self.set(7,Register8bit::H),
            0xfd => self.set(7,Register8bit::L),
            0xfe => self.set(7,Register8bit::HlDirectMemory),
            0xff => self.set(7,Register8bit::A),
            _=> panic!("{code} isnt a real cb call")
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::register::Registers;
    const TEST_SP_ADDR: u16 = 0x000F;
    fn create_cpu() -> Cpu {
        Cpu {
            ime: false,
            clock: Clock { m: 0, t: 0 },
            registers: Registers::default(),
            memory: Memory::new(),
            halted: false,
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
        cpu.pop_register16bit(Register16bit::BC);

        assert_eq!(cpu.registers.get_bc(), 0x1234);
        assert_eq!(cpu.registers.sp, TEST_SP_ADDR.wrapping_add(2));
        assert_eq!(cpu.clock.m, 12);
    }

    #[test]
    fn test_pop_de() {
        let mut cpu = create_cpu();
        cpu.registers.sp = TEST_SP_ADDR;
        cpu.memory.write_word(TEST_SP_ADDR, 0x5678);
        cpu.pop_register16bit(Register16bit::DE);

        assert_eq!(cpu.registers.get_de(), 0x5678);
        assert_eq!(cpu.registers.sp, TEST_SP_ADDR.wrapping_add(2));
        assert_eq!(cpu.clock.m, 12);
    }

    #[test]
    fn test_pop_hl() {
        let mut cpu = create_cpu();
        cpu.registers.sp = TEST_SP_ADDR;
        cpu.memory.write_word(TEST_SP_ADDR, 0x9ABC);
        cpu.pop_register16bit(Register16bit::HL);

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
        cpu.pop_register16bit(Register16bit::SP);
    }

    #[test]
    fn test_pop_af() {
        let mut cpu = create_cpu();
        cpu.registers.sp = TEST_SP_ADDR;
        cpu.memory.write_word(TEST_SP_ADDR, 0x5678);
        cpu.pop_af();
        assert_eq!(cpu.registers.a, 0x56);

        assert_eq!(cpu.registers.get_af(), 0x5678); // Assuming you have a get_af() method
        assert_eq!(cpu.registers.sp, TEST_SP_ADDR.wrapping_add(2));
        assert_eq!(cpu.clock.m, 12);
    }
}
