#[inline]
#[cfg_attr(kani, kani::requires(buf.len() >= 2 && ib <= buf.len() - 2))]
pub fn r_u16_be(buf: &[u8], ib: usize) -> u16 {
    let lo = buf[ib+1]; // ib+1 checked first
    u16::from_be_bytes([buf[ib], lo])
}
#[inline]
#[cfg_attr(kani, kani::requires(buf.len() >= 2 && ib <= buf.len() - 2))]
pub fn r_u16_le(buf: &[u8], ib: usize) -> u16 {
    let hi = buf[ib+1];
    u16::from_le_bytes([buf[ib], hi])
}
#[inline]
#[cfg_attr(kani, kani::requires(buf.len() >= 2 && ib <= buf.len() - 2))]
pub fn r_i16_le(buf: &[u8], ib: usize) -> i16 {
    let hi = buf[ib+1];
    i16::from_le_bytes([buf[ib], hi])
}
#[inline]
#[cfg_attr(kani, kani::requires(buf.len() >= 4 && ib <= buf.len() - 4))]
pub fn r_u32_le(buf: &[u8], ib: usize) -> u32 {
    let b3 = buf[ib+3];
    u32::from_le_bytes([buf[ib], buf[ib+1], buf[ib+2], b3])
}
#[inline]
#[cfg_attr(kani, kani::requires(buf.len() >= 4 && ib <= buf.len() - 4))]
pub fn r_i32_le(buf: &[u8], ib: usize) -> i32 {
    let b3 = buf[ib+3];
    i32::from_le_bytes([buf[ib], buf[ib+1], buf[ib+2], b3])
}
#[inline]
#[cfg_attr(kani, kani::requires(buf.len() >= 4 && ib <= buf.len() - 4))]
pub fn r_f32_le(buf: &[u8], ib: usize) -> f32 {
    let b3 = buf[ib+3];
    f32::from_le_bytes([buf[ib], buf[ib+1], buf[ib+2], b3])
}

macro_rules! proof_reader {
    ($harness:ident, $reader:ident) => {
        #[cfg(kani)]
        #[kani::proof_for_contract($reader)]
        fn $harness() {
            let bytes: [u8; 128] = kani::any();
            let slice = kani::slice::any_slice_of_array(&bytes);
            let ib: usize = kani::any();
            let _ = $reader(slice, ib);
        }
    }
}

proof_reader!(contract_r_u16_le, r_u16_le);
proof_reader!(contract_r_u16_be, r_u16_be);
proof_reader!(contract_r_i16_be, r_i16_le);
proof_reader!(contract_r_f32_le, r_f32_le);
proof_reader!(contract_r_u32_be, r_u32_le);
proof_reader!(contract_r_i32_le, r_i32_le);