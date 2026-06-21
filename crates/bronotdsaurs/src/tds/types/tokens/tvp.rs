use crate::tds::prelude::*;

span!(
    TVPTypeNameSpan,
    TVPOrderUniqueSpan,
    TVPColumnOrderingSpan,
    TVPOrderingUniqueItemSpan,
    TVPColumnOrderingItemSpan,
);

/// 2.2.5.5.5 Table Valued Parameter (TVP) Values
#[derive(Debug, Clone)]
pub struct TableValuedParameterSpan<'a> {
    pub(super) bytes: &'a [u8],
    typename: TVPTypeNameSpan<'a>,
    col_metadata: ColMetaDataSpan<'a>,
    order_unique: Option<TVPOrderUniqueSpan<'a>>,
    column_ordering: Option<TVPColumnOrderingSpan<'a>>,
    ib_rows: usize,
}

impl<'a> TableValuedParameterSpan<'a> {
    pub const TVPTYPE: u8 = 0xf3;
    pub const TVP_COLUMN_ORDERING_TOKEN: u8 = 0x11;
    pub const TVP_ROW_TOKEN: u8 = 0x10;

    pub fn new(buffer: &'a [u8]) -> Result<Self, DecodeError> {
        let typename = TVPTypeNameSpan { bytes: buffer };
        let ib_col_metadata = typename.ib_type_name() + typename.cch_type_name() * 2;
        let col_metadata = ColMetaDataSpan::new(&buffer[ib_col_metadata..])?;
        let mut cursor = ib_col_metadata + col_metadata.bytes.len();
        let mut order_unique = None;
        let mut column_ordering = None;

        while cursor < buffer.len() {
            match buffer[cursor] {
                Self::TVP_ROW_TOKEN => {
                    let size = 2 + TVPOrderUniqueSpan {
                        bytes: &buffer[cursor + 1..],
                    }
                    .into_iter()
                    .count()
                        * 3;
                    let item = TVPOrderUniqueSpan {
                        bytes: &buffer[cursor + 1..cursor + 1 + size],
                    };
                    cursor += 1 + size;
                    order_unique = Some(item);
                }
                Self::TVP_COLUMN_ORDERING_TOKEN => {
                    let size = 2 + TVPColumnOrderingSpan {
                        bytes: &buffer[cursor + 1..],
                    }
                    .into_iter()
                    .count()
                        * 2;
                    let item = TVPColumnOrderingSpan {
                        bytes: &buffer[cursor + 1..cursor + 1 + size],
                    };
                    cursor += 1 + size;
                    column_ordering = Some(item);
                }
                _ => break,
            }
        }
        let ib_rows = cursor;

        Ok(Self {
            bytes: buffer,
            typename,
            col_metadata,
            order_unique,
            column_ordering,
            ib_rows,
        })
    }

    #[inline(always)]
    pub fn typename(&self) -> TVPTypeNameSpan<'a> {
        self.typename
    }

    #[inline(always)]
    pub fn col_metadata(&self) -> ColMetaDataSpan<'a> {
        self.col_metadata.clone()
    }

    #[inline(always)]
    pub fn order_unique(&self) -> Option<TVPOrderUniqueSpan<'a>> {
        self.order_unique
    }

    #[inline(always)]
    pub fn column_ordering(&self) -> Option<TVPColumnOrderingSpan<'a>> {
        self.column_ordering
    }

    #[inline(always)]
    pub fn rows(&self) -> &'a [u8] {
        &self.bytes[self.ib_rows..]
    }
}

#[derive(Debug, Clone, Copy)]
pub struct TableValuedParameterSpanIter<'a> {
    bytes: &'a [u8],
    col_metadata: &'a ColMetaDataSpan<'a>,
}

impl<'a> TableValuedParameterSpanIter<'a> {
    const TVP_ROW_TOKEN: u8 = 0x01;
    const TVP_END_TOKEN: u8 = 0x00;
}

impl<'a> IntoIterator for &'a TableValuedParameterSpan<'a> {
    type Item = RowSpanIter<'a>;
    type IntoIter = TableValuedParameterSpanIter<'a>;

    fn into_iter(self) -> Self::IntoIter {
        TableValuedParameterSpanIter {
            bytes: &self.bytes[self.ib_rows..],
            col_metadata: &self.col_metadata,
        }
    }
}

impl<'a> Iterator for TableValuedParameterSpanIter<'a> {
    type Item = RowSpanIter<'a>;

    fn next(&mut self) -> Option<RowSpanIter<'a>> {
        if self.bytes.is_empty() {
            return None;
        }
        match self.bytes[0] {
            Self::TVP_END_TOKEN => return None,
            Self::TVP_ROW_TOKEN => {}
            _ => return None,
        }
        let mut cursor = 1;
        for i in 0..self.col_metadata.count() {
            let s = self.col_metadata.stride(i);
            cursor += walk(self.bytes, cursor, s)?;
        }
        if cursor > self.bytes.len() {
            return None;
        }
        let row = RowSpanIter::new(&self.bytes[1..cursor], self.col_metadata);
        self.bytes = &self.bytes[cursor..];
        Some(row)
    }
}

impl<'a> TVPTypeNameSpan<'a> {
    pub fn db_name(&self) -> NVarCharSpan<'a> {
        NVarCharSpan {
            bytes: &self.bytes[self.ib_db_name()..self.ib_db_name() + self.cch_db_name() * 2],
        }
    }

    #[inline(always)]
    const fn ib_db_name(&self) -> usize {
        1
    }

    #[inline(always)]
    fn cch_db_name(&self) -> usize {
        self.bytes[0].into()
    }

    #[inline(always)]
    pub fn owning_schema(&self) -> NVarCharSpan<'a> {
        NVarCharSpan {
            bytes: &self.bytes
                [self.ib_owning_schema()..self.ib_owning_schema() + self.cch_owning_schema() * 2],
        }
    }

    #[inline(always)]
    fn ib_owning_schema(&self) -> usize {
        self.ib_db_name() + self.cch_db_name() * 2 + 1
    }

    #[inline(always)]
    fn cch_owning_schema(&self) -> usize {
        self.bytes[self.ib_db_name() + self.cch_db_name() * 2].into()
    }

    #[inline(always)]
    pub fn type_name(&self) -> NVarCharSpan<'a> {
        NVarCharSpan {
            bytes: &self.bytes[self.ib_type_name()..self.ib_type_name() + self.cch_type_name() * 2],
        }
    }

    #[inline(always)]
    fn ib_type_name(&self) -> usize {
        self.ib_owning_schema() + self.cch_owning_schema() * 2 + 1
    }

    #[inline(always)]
    fn cch_type_name(&self) -> usize {
        self.bytes[self.ib_owning_schema() + self.cch_owning_schema() * 2].into()
    }
}

impl<'a> TVPOrderingUniqueItemSpan<'a> {
    pub const LENGTH: usize = 3;

    #[inline(always)]
    pub fn col_num(&self) -> u16 {
        r_u16_le(self.bytes, 0)
    }

    #[inline(always)]
    pub fn f_order_asc(&self) -> bool {
        (self.bytes[2] >> 1) & 0x1 == 1
    }

    #[inline(always)]
    pub fn f_order_desc(&self) -> bool {
        (self.bytes[2] >> 2) & 0x1 == 1
    }

    #[inline(always)]
    pub fn f_unique(&self) -> bool {
        (self.bytes[2] >> 3) & 0x1 == 1
    }
}

#[derive(Debug, Clone, Copy)]
pub struct TVPOrderUniqueSpanIter<'a> {
    bytes: &'a [u8],
    remaining: usize,
}

impl<'a> TVPOrderUniqueSpan<'a> {
    #[inline(always)]
    fn count(&self) -> usize {
        r_u16_le(self.bytes, 0) as usize
    }
}

impl<'a> IntoIterator for TVPOrderUniqueSpan<'a> {
    type Item = TVPOrderingUniqueItemSpan<'a>;
    type IntoIter = TVPOrderUniqueSpanIter<'a>;

    fn into_iter(self) -> Self::IntoIter {
        TVPOrderUniqueSpanIter {
            bytes: &self.bytes[2..],
            remaining: self.count(),
        }
    }
}

impl<'a> Iterator for TVPOrderUniqueSpanIter<'a> {
    type Item = TVPOrderingUniqueItemSpan<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.remaining == 0 || self.bytes.len() < TVPOrderingUniqueItemSpan::LENGTH {
            debug_assert!(
                self.remaining == 0,
                "unexpected self.remaining. should be 0, got {}",
                self.remaining
            );
            return None;
        }
        let item = Some(TVPOrderingUniqueItemSpan {
            bytes: &self.bytes[..TVPOrderingUniqueItemSpan::LENGTH],
        });
        self.bytes = &self.bytes[TVPOrderingUniqueItemSpan::LENGTH..];
        self.remaining -= 1;
        item
    }
}

impl<'a> TVPColumnOrderingSpan<'a> {
    #[inline(always)]
    fn count(&self) -> usize {
        r_u16_le(self.bytes, 0) as usize
    }
}

impl<'a> TVPColumnOrderingItemSpan<'a> {
    const LENGTH: usize = 3;

    #[inline(always)]
    pub fn col_num(&self) -> u16 {
        r_u16_le(self.bytes, 0)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct TVPColumnOrderingSpanIter<'a> {
    bytes: &'a [u8],
    remaining: usize,
}

impl<'a> IntoIterator for TVPColumnOrderingSpan<'a> {
    type Item = TVPColumnOrderingItemSpan<'a>;
    type IntoIter = TVPColumnOrderingSpanIter<'a>;

    fn into_iter(self) -> Self::IntoIter {
        TVPColumnOrderingSpanIter {
            bytes: &self.bytes[2..],
            remaining: self.count(),
        }
    }
}

impl<'a> Iterator for TVPColumnOrderingSpanIter<'a> {
    type Item = TVPColumnOrderingItemSpan<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.remaining == 0 || self.bytes.len() < TVPColumnOrderingItemSpan::LENGTH {
            debug_assert!(
                self.remaining == 0,
                "unexpected self.remaining. should be 0, got {}",
                self.remaining
            );
            return None;
        }
        let item = Some(TVPColumnOrderingItemSpan {
            bytes: &self.bytes[..TVPColumnOrderingItemSpan::LENGTH],
        });
        self.bytes = &self.bytes[TVPColumnOrderingItemSpan::LENGTH..];
        self.remaining -= 1;
        item
    }
}
