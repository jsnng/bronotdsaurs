use crate::tds::prelude::*;

/// Used to decode a row span with the provided column metadata.
#[derive(Debug, Clone, Copy)]
pub struct RowSpanIter<'a> {
    pub bytes: &'a [u8],
    col_metadata_iter: ColumnMetaDataSpanIter<'a>,
    null_bitmap: &'a [u8],
    column: usize,
}

impl<'a> RowSpanIter<'a> {
    pub fn new(bytes: &'a [u8], col_metadata: &'a ColMetaDataSpan<'a>) -> Self {
        Self {
            bytes,
            col_metadata_iter: col_metadata.into_iter(),
            null_bitmap: &[],
            column: 0,
        }
    }

    pub fn from_owned(bytes: &'a [u8], col_metadata: &'a ColMetaDataOwned) -> Self {
        debug_assert!(matches!(bytes.first(), Some(0xd1 | 0xd2)));
        // Sized on the same basis as `NbcRow::steps`, which framed this token.
        let bitmap = match bytes.first() {
            Some(0xd2) => col_metadata.strides_as_slice().len().div_ceil(8),
            _ => 0,
        };
        Self {
            bytes: bytes.get(1 + bitmap..).unwrap_or_default(),
            col_metadata_iter: col_metadata.into_iter(),
            null_bitmap: bytes.get(1..1 + bitmap).unwrap_or_default(),
            column: 0,
        }
    }

    pub fn remaining(&self) -> &'a [u8] {
        self.bytes
    }
}

impl<'a> Iterator for RowSpanIter<'a> {
    type Item = RowItemSpan<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        let col = self.col_metadata_iter.next()?;
        let column = self.column;
        self.column += 1;

        if self
            .null_bitmap
            .get(column / 8)
            .is_some_and(|b| b >> (column % 8) & 1 == 1)
        {
            return Some(RowItemSpan { bytes: &[] });
        }

        let bytes = to_dtype_bytes(&mut self.bytes, col.ty())?;
        Some(RowItemSpan { bytes })
    }
}
