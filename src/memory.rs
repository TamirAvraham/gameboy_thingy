use std::panic::Location;

use crate::helper::{combine_u8_to_u16, split_u16};

pub const MEM_SIZE:usize=16 * 1024; // 16kb
pub const BANK_SIZE:usize=16*1024; //16kb just like MEM_SIZE

pub struct Memory{
    mem:[u8; MEM_SIZE], // not really neccery beacause im dividing the memory into "banks" each 16kb, memory should be 64k
    _inbios:i16,
    _bios:[u8;BANK_SIZE/64],
    _rom: [u8;BANK_SIZE],
    _wram: [u8;BANK_SIZE/2],
    _eram: [u8;BANK_SIZE/2],
    _zram: [u8;BANK_SIZE/128]
}

impl Memory {
    pub fn new()->Self{
        Self{
             // Flag indicating BIOS is mapped in
             _inbios : 1,
            _bios:[0;MEM_SIZE/64],
            mem:[0;MEM_SIZE],
            _wram: [0;BANK_SIZE/2],
            _eram: [0;BANK_SIZE/2],
            _zram: [0;BANK_SIZE/128],
            _rom:[0;BANK_SIZE]
        }
    }
    pub fn write_byte(&mut self,location:u16,byte:u8){
        match location {
            0x0000..=0x00FF => { //bios case.
                self._bios[location as usize] = byte;
            }
            0x0100..=0x3FFF =>{ //rom case
                self._rom[location as usize] = byte;
            }
            0x8000..=0x9FFF=>{ //gpu thing,which we don't yet have
                //pass for now
            }
            0xA000..=0xBFFF=>{
                self._eram[location as usize] = byte;
            }
            0xC000..=0xFDFF=>{ //Don't know yet if i can merge the Working ram with it's shadow, but for now it's what I've have done
                self._wram[location as usize] = byte;
            }
            0xFF80..=0xFFFF=>{
                self._zram[location as usize] = byte;
            }
            //Didn't make cases for sprite information , Memory mapping i/o .. will be adding later on.
            _=>{
                //default will pass
            }

        }
        self.mem[location as usize]=byte;
    }
    pub fn read_byte(&self, location:u16)->u8{
        //self.mem[location as usize] 
        match location & 0xF000 {
            0x0000=>{ 
                if self._inbios ==1 { //if the bios is still mapped we want to return what's in the bios
                    if location < 0x1000 {
                        return self._bios[location as usize];
                    }else{
                        return 0; //missing something here that i can't implement now
                    } 
                } else{
                    return self._rom[location as usize];
                }
            }
            
            0x3000 | 0x7000 =>{
                return self._rom[location as usize];
            }
            0xD000 | 0xE000 =>{
                return self._wram[(location&0x1FFF) as usize];
            }
            0xF000 =>{
                match location&0x0FFF{
                    0xD00=>{
                        return self._wram[(location & 0x1FFF) as usize];
                    }
                    0xF00=>{
                        if location >=0xFF80 { // Zero ram location
                            return self._zram[(location&0x7F) as usize];
                        } else {
                            // I/O control handling
			                // Currently unhandled
                            return 0;
                        }
                    }
                    _=> return 0
                }
            }
            _=>return 0
            
        }
    }
    pub fn read_word(&self,location:u16)->u16{
        combine_u8_to_u16(self.read_byte(location),self.read_byte(location.wrapping_add(1)))
    }
    pub fn write_word(&mut self, location:u16, word:u16){
        let (first,second)=split_u16(word);
        self.write_byte(location,first);
        self.write_byte(location.wrapping_add(1),second);
    }
    pub fn get_refm_to_byte(&mut self,location:u16)->&mut u8{
        &mut self.mem[location as usize]
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::helper::{combine_u8_to_u16, split_u16}; // Assuming you have these methods in the helper module

    #[test]
    fn test_memory_new() {
        let memory = Memory::new();
        for byte in memory.mem.iter() {
            assert_eq!(*byte, 0);
        }
    }

    #[test]
    fn test_write_byte() {
        let mut memory = Memory::new();
        memory.write_byte(0x0001, 0x12);
        assert_eq!(memory.mem[0x0001], 0x12);
    }

    #[test]
    fn test_read_byte() {
        let mut memory = Memory::new();
        memory.write_byte(0x0001, 0x34);
        assert_eq!(memory.read_byte(0x0001), 0x34);
    }

    #[test]
    fn test_write_word() {
        let mut memory = Memory::new();
        memory.write_word(0x0002, 0x5678);
        assert_eq!(memory.mem[0x0002], 0x56);
        assert_eq!(memory.mem[0x0003], 0x78);
    }

    #[test]
    fn test_read_word() {
        let mut memory = Memory::new();
        memory.write_word(0x0002, 0x9ABC);
        assert_eq!(memory.read_word(0x0002), 0x9ABC);
    }

    #[test]
    fn test_read_write_consistency() {
        let mut memory = Memory::new();
        memory.write_byte(0x0004, 0xDE);
        assert_eq!(memory.read_byte(0x0004), 0xDE);

        memory.write_word(0x0006, 0xF012);
        assert_eq!(memory.read_word(0x0006), 0xF012);
    }
}
