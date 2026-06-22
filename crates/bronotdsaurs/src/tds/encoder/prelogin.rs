use crate::tds::prelude::*;
use crate::tds::session::prelude::*;

#[cfg(kani)]
use kani;

impl MessageEncoder for PreLoginPacket {
    type Error = EncodeError;
    type Header = PreLoginHeader;

    fn oneshot(
        &self,
        buf: &mut SessionBuffer,
        header: &mut Self::Header,
    ) -> Result<usize, Self::Error> {
        let thread_id_bytes = self.thread_id.map(|x| x.to_be_bytes());
        let mut opts: [(PLOptionType, &[u8]); 5] = [
            (PLOptionType::Version, &self.version),
            (PLOptionType::Encryption, self.encryption.as_slice()),
            (
                PLOptionType::InstOpt,
                self.inst_opt.as_deref().unwrap_or(&[]),
            ),
            (
                PLOptionType::ThreadId,
                thread_id_bytes.as_ref().map(|x| x as &[u8]).unwrap_or(&[]),
            ),
            (PLOptionType::Mars, self.mars.as_slice()),
        ];

        let mut n = 0;
        for i in 0..5 {
            if !opts[i].1.is_empty() {
                opts[n] = opts[i];
                n += 1;
            }
        }

        let buffer = buf.writeable();

        let opts = &opts[..n];

        debug_assert_eq!(opts[0].0, PLOptionType::Version);

        let descriptors_length = opts.len() * PreLoginPacket::OPT_LENGTH + 1;
        let mut offset = descriptors_length;
        let mut cursor = PreLoginHeader::LENGTH;

        for (token, bytes) in opts {
            let len = bytes.len() as u16;
            buffer[cursor..cursor + PreLoginPacket::OPT_LENGTH].copy_from_slice(&[
                *token as u8,
                (offset >> 8) as u8,
                (offset & 0xff) as u8,
                (len >> 8) as u8,
                (len & 0xff) as u8,
            ]);
            cursor += PreLoginPacket::OPT_LENGTH;
            offset += bytes.len();
        }
        buffer[cursor] = PLOptionType::Terminator as u8;

        cursor = PreLoginHeader::LENGTH + descriptors_length;
        for (_, bytes) in opts {
            buffer[cursor..cursor + bytes.len()].copy_from_slice(bytes);
            cursor += bytes.len();
        }

        header.length = cursor as u16;
        buffer[0..PreLoginHeader::LENGTH].copy_from_slice(&header.as_bytes());
        Ok(cursor)
    }
}

#[cfg(kani)]
#[kani::proof]
#[kani::unwind(8)]
fn proof_opts_version_first_terminator_last() {
    let mut buf = SessionBuffer::default();
    buf.set_buffer_maximum_size(64).unwrap();

    let packet = PreLoginPacketBuilder::default()
        .version(kani::any())
        .build()
        .unwrap();

    let mut header = PreLoginHeader::default();
    let len = packet.oneshot(&mut buf, &mut header).unwrap();
    buf.tail(len).unwrap();
    kani::assume(len > PreLoginHeader::LENGTH);
    assert_eq!(
        buf.readable()[PreLoginHeader::LENGTH],
        PLOptionType::Version
    );
    // let terminator = buf[PreLoginHeader::LENGTH + PreLoginPacket::OPT_LENGTH];

    let mut cursor = PreLoginHeader::LENGTH;
    while cursor < len && buf.readable()[cursor] != PLOptionType::Terminator {
        cursor += PreLoginPacket::OPT_LENGTH;
    }

    assert!(cursor < len);
    assert_eq!(buf.readable()[cursor], PLOptionType::Terminator)
}

#[cfg(all(feature = "std", not(feature = "tds8.0"), test))]
mod tests {
    extern crate std;
    use crate::tds::encoder::traits::MessageEncoder;
    use crate::tds::session::prelude::*;
    use crate::tds::types::prelude::*;

    #[test]
    fn test_prelogin_encode() {
        let prelogin = PreLoginPacketBuilder::default().build().unwrap();

        let mut buf = SessionBuffer::default();
        let mut header = PreLoginHeader::default();

        let n = prelogin
            .oneshot(&mut buf, &mut header)
            .expect("prelogin.encode() failed?");
        let _n = buf.tail(n);
        println!("{}", n);
        let readable = &buf.readable()[..n];
        println!("{:?}", readable);

        for (i, chunk) in readable.chunks(16).enumerate() {
            let hex: String = chunk.iter().map(|b| format!("{:02x} ", b)).collect();
            println!("{:03}: {:<48} ", i * 16, hex);
        }

        let bytes: &[u8; 8] = buf.readable()[0..8].try_into().unwrap();

        let header = PreLoginHeader::from_bytes(bytes);
        println!("{}", header);
        assert_eq!(header.ty, ClientMessageType::PreLogin);
        // assert_eq!(header.length, n as u16 + PreLoginHeader::LENGTH as u16);
        assert_eq!(buf.readable()[0], ClientMessageType::PreLogin);
        println!("SESSION BUFFER\n{}", buf);
    }
}
