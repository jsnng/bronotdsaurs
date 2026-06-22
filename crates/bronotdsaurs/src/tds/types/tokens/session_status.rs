use crate::tds::prelude::*;

impl<'a> SessionStatusSpan<'a> {
    pub fn new(bytes: &'a [u8]) -> Result<Self, DecodeError> {
        if bytes.len() < 10 {
            return Err(DecodeError::InvalidData("".to_string()))
        }

        let length =  r_u32_le(bytes, 1);
        if bytes.len() != 5 + length as usize {
            return Err(DecodeError::InvalidData("".to_string()));
        }

        Ok(Self { bytes })
    }

    pub fn ty(&self) -> u8 {
        self.bytes[0]
    }

    pub fn length(&self) -> u32 {
        r_u32_le(self.bytes, 1)
    }

    pub fn seq_no(&self) -> u32 {
        r_u32_le(self.bytes, 5)
    }

    pub fn status(&self) -> u8{
        self.bytes[9]
    }

    pub fn session_state_dataset(&self) -> &'a [u8] {
        &self.bytes[10..]
    }
}

impl<'a> IntoIterator for &'a SessionStatusSpan<'a> {
    type Item = SessionStateDataRef<'a>;

    type IntoIter = SessionStatusIter<'a>;

    fn into_iter(self) -> Self::IntoIter {
        SessionStatusIter { bytes: self.session_state_dataset(),  cursor: 0}
    }
}

#[derive(Debug, Clone, Copy)]
 pub struct SessionStateDataRef<'a> {
      pub state_id: u8,
      pub state_value: &'a [u8],
  }

#[derive(Debug, Clone, Copy, Default)]
pub struct SessionStatusIter<'a> {
    pub(crate) bytes: &'a [u8],
    pub(crate) cursor: usize,
}

impl<'a> Iterator for SessionStatusIter<'a> {
    type Item = SessionStateDataRef<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.cursor >= self.bytes.len() { return None; }
        let state_id = self.bytes[self.cursor];
        self.cursor += 1;

        let length = *self.bytes.get(self.cursor)?;
        self.cursor += 1;

        let length = if length == 0xff {
            let n = r_u32_le(self.bytes, self.cursor) as usize;
            self.cursor += 4;
            n
        } else {
            length as usize
        };

        let val = self.bytes.get(self.cursor..self.cursor+length)?;
        self.cursor += length;
        Some(SessionStateDataRef { state_id, state_value: val})
    }
}

#[derive(Debug, Clone)]
pub struct SessionStatusToken {
    pub(crate) ty: u8,
    pub(crate) length: u32,
    pub(crate) seq_no: u32,
    pub(crate) status: u8,
    pub(crate) session_state_dataset: SessionStatusDataset,
}

#[derive(Debug, Clone)]
pub struct SessionStatusDataset(pub Vec<SessionStatusData>);

#[derive(Debug, Clone)]
pub struct SessionStatusData {
    pub(crate) state_id: u8,
    pub(crate) state_value: Vec<u8>,
}
