pub struct Registers{
    pub a:u8,
    pub b:u8,
    pub c:u8,
    pub d:u8,
    pub e:u8,
    f:u8,
    pub h:u8,
    pub l:u8,
    pub pc:u8,
    pub sp:u8
}
pub enum Flags{
    Z=0b10000000,
    N=0b01000000,
    H=0b00100000,
    C=0b00010000,
}
impl Registers {
    #[inline]
    fn combine_u8_to_u16(f:u8,s:u8)->u16{

        ((f as u16)>>8) | s as u16
    }
    #[inline]
    fn write_u16_into_two_u8(v:u16,f: &mut u8,s: &mut u8){
        let [first,second]=v.to_be_bytes();
        *f=first;
        *s=second;
    }
    pub fn get_bc(&self)->u16{
        Self::combine_u8_to_u16(self.b,self.c)
    }
    pub fn get_de(&self)->u16{
        Self::combine_u8_to_u16(self.d,self.e)
    }
    pub fn get_hl(&self)->u16{
        Self::combine_u8_to_u16(self.h,self.l)
    }
    pub fn write_bc(&mut self,value:u16)->u16{
        Self::write_u16_into_two_u8(value, &mut self.b, &mut self.c);
        value
    }
    pub fn write_de(&mut self,value:u16)->u16{
        Self::write_u16_into_two_u8(value, &mut self.d, &mut self.e);
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