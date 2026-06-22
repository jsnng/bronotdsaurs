use crate::tds::prelude::*;

#[derive(Debug, Clone)]
pub struct OrderToken {
    pub(crate) length: u16,
    pub(crate) col_num: Vec<u16>,
}

impl<'a> OrderSpan<'a> {

    pub fn new(bytes: &'a [u8]) -> Result<Self, DecodeError> {
     if bytes.len() < 3 {
            return Err(kani_error_stubbed!(DecodeError::InvalidData("".to_string())));
        }
        let length = r_u16_le(bytes, 1);
        if bytes.len() != 3 + length as usize || !length.is_multiple_of(2){
            return Err(kani_error_stubbed!(DecodeError::InvalidData("".to_string())));
        }
        Ok(Self { bytes })
    }

    pub fn ty(&self) -> u8 {
        self.bytes[0]
    }

    pub fn length(&self) -> u16 {
        r_u16_le(self.bytes, 1)
    }
}

impl<'a> IntoIterator for &'a OrderSpan<'a> {
    type Item = u16;

    type IntoIter = OrderSpanIter<'a>;

    fn into_iter(self) -> Self::IntoIter {
        OrderSpanIter::new(&self.bytes[3..], self.length() as usize / 2)
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct OrderSpanIter<'a> {
    pub(crate) bytes: &'a [u8],
    pub(crate) remaining: usize,
}

impl<'a> OrderSpanIter<'a> {
    #[inline(always)]
    pub fn new(bytes: &'a [u8], remaining: usize) -> Self {
        Self { bytes, remaining }
    }
}

impl<'a> Iterator for OrderSpanIter<'a> {
    type Item = u16;
    
    fn next(&mut self) -> Option<Self::Item> {
        if self.remaining == 0 {
            return None;
        }
        let idx = self.bytes.len() - self.remaining*2;
        let item = r_u16_le(self.bytes, idx);
        self.remaining -= 1;
        Some(item)

    } 
}
