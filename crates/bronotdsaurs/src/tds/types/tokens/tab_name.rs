use crate::tds::prelude::*;

#[derive(Debug, Clone, Builder)]
#[builder(no_std, setter(strip_option))]
pub struct TabNameToken {
    ty: u8,
    length: u16,
    items: Vec<TabNameTokenItem>,
}

#[derive(Debug, Clone)]
pub struct TabNameTokenItem {
    pub(crate) parts: Vec<String>
}

impl<'a> TabNameSpan<'a> {
    pub fn new(bytes: &'a [u8]) -> Result<Self, DecodeError> {
        if bytes.len() < 3 { return Err(kani_error_stubbed!(DecodeError::InvalidData("TabNameSpan self.bytes < 3".to_string()))) }
        Ok(Self { bytes })
    }
    pub fn ty(&self) -> u8 {
        self.bytes[0]
    }
    pub fn length(&self) -> u16 {
        r_u16_le(self.bytes, 1)
    }
}

impl<'a> TabNameSpan<'a> {
    pub const FIXED_SPAN_OFFSET: usize = 3;
}

impl<'a> IntoIterator for &'a TabNameToken {
      type Item = &'a TabNameTokenItem;
      type IntoIter = core::slice::Iter<'a, TabNameTokenItem>;

      fn into_iter(self) -> Self::IntoIter {
          self.items.iter()
      }
  }

impl<'a> IntoIterator for TabNameSpan<'a> {
    type Item = TabNameSpanItem<'a>;

    type IntoIter = TabNameSpanIter<'a>;

    fn into_iter(self) -> Self::IntoIter {
        TabNameSpanIter {
            bytes: &self.bytes[Self::FIXED_SPAN_OFFSET..],
        }
    }
}

impl<'a> IntoIterator for &'a TabNameSpan<'a> {
    type Item = TabNameSpanItem<'a>;

    type IntoIter = TabNameSpanIter<'a>;

    fn into_iter(self) -> Self::IntoIter {
        TabNameSpanIter {
            bytes: &self.bytes[TabNameSpan::FIXED_SPAN_OFFSET..],
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct TabNameSpanItem<'a> {
    bytes: &'a [u8],
    num_parts: usize,
    ib_parts: usize,
}

#[derive(Debug, Clone, Copy)]
pub struct TabNameSpanItemIter<'a> {
    bytes: &'a [u8],
    num_parts: usize,
}

impl TabNameTokenItem {
    pub fn parts(&self) -> &[String] {
        &self.parts
    }
}

impl<'a> IntoIterator for TabNameSpanItem<'a> {
    type Item = NVarCharSpan<'a>;

    type IntoIter = TabNameSpanItemIter<'a>;

    fn into_iter(self) -> Self::IntoIter {
        TabNameSpanItemIter {
            bytes: &self.bytes[self.ib_parts..],
            num_parts: self.num_parts,
        }
    }
}

impl<'a> Iterator for TabNameSpanItemIter<'a> {
    type Item = NVarCharSpan<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.num_parts == 0 {
            return None;
        }
        let cch_tab_name_item_part: usize = r_u16_le(self.bytes, 0) as usize;
        let end = 2 + cch_tab_name_item_part * 2;
        let item = NVarCharSpan::new(&self.bytes[2..end]);
        self.bytes = &self.bytes[end..];
        self.num_parts -= 1;
        Some(item)
    }
}

impl<'a> Iterator for TabNameSpanIter<'a> {
    type Item = TabNameSpanItem<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.bytes.is_empty() { return None }
        let num_parts = self.bytes[0] as usize;
        let mut cursor = 1usize;
        for _ in 0..num_parts {
            if cursor + 2 > self.bytes.len() { return None; }
            let part_length = r_u16_le(self.bytes, cursor) as usize;
            cursor += 2 + part_length * 2;
        }
        if cursor > self.bytes.len() { return None; }

        let item  = TabNameSpanItem {
            bytes: &self.bytes[..cursor],
            num_parts,
            ib_parts: 1,
        };

        self.bytes = &self.bytes[cursor..];
        Some(item)
    }
}
