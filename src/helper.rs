#[inline(always)]
pub fn combine_u8_to_u16(f:u8,s:u8)->u16{

    ((f as u16)<<8) | s as u16
}
#[inline(always)]
pub fn split_u16(v:u16)->(u8,u8){
    let [first,second]=v.to_be_bytes();
    (first,second)
}