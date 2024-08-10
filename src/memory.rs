use crate::helper::{combine_u8_to_u16, split_u16};

pub const MEM_SIZE:usize=16 * 1024; // 16kb
pub struct Memory{
    mem:[u8; MEM_SIZE]
}

impl Memory {
    pub fn new()->Self{
        Self{
            mem:[0;MEM_SIZE]
        }
    }
    pub fn write_byte(&mut self,location:u16,byte:u8){
        self.mem[location as usize]=byte;
    }
    pub fn read_byte(&self, location:u16)->u8{
        self.mem[location as usize]
    }
    pub fn read_word(&self,location:u16)->u16{
        combine_u8_to_u16(self.read_byte(location),self.read_byte(location.wrapping_add(1)))
    }
    pub fn write_word(&mut self, location:u16, word:u16){
        let (first,second)=split_u16(word);
        self.write_byte(location,first);
        self.write_byte(location.wrapping_add(1),second);
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
