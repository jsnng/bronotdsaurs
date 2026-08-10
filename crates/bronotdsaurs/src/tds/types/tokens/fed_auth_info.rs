use crate::tds::prelude::*;

span!(FedAuthSpan);

#[derive(Debug, Clone, Copy)]
pub struct FedAuthInfoToken {

}

impl<'a> FedAuthSpan<'a> {
    pub const FIXED_SPAN_SIZE: usize = 9;
    pub fn new(bytes: &'a [u8]) -> Result<Self, DecodeError> {
        if bytes.len() < Self::FIXED_SPAN_SIZE { return Err(DecodeError::InvalidData("".to_string())) }
        let token_length = r_u32_le(bytes, 1);
        let count_of_info_ids = r_u32_le(bytes, 5);


        todo!()
    }
}

struct FedAuthSpanIter<'a>  {
    bytes: &'a [u8],
    remaining: usize,
}

// impl<'a> IntoIterator for &'a FedAuthSpan<'a> {
//     type Item = (u8, &'a [u8]);
//     type IntoIter = FedAuthSpanIter<'a>;

//     fn into_iter(self) -> Self::IntoIter {
//         FedAuthSpanIter::new(self.bytes, self.count_of_info_ids() as usize)
//     }
// }

// impl<'a> Iterator for FedAuthSpanIter<'a> {
//     type Item;

//     fn next(&mut self) -> Option<Self::Item> {
//         todo!()
//     }
// }


enum FedAuthInfoId {
    StsUrl = 0x01,
    Spn = 0x02,
}