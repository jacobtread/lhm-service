use tokio_util::{
    bytes::{Buf, BufMut, Bytes, BytesMut},
    codec::{Decoder, Encoder},
};

pub struct LHMFrame {
    pub id: u32,
    pub body: Bytes,
}

pub struct LHMFrameHeader {
    length: u32,
    id: u32,
}

impl LHMFrameHeader {
    pub fn try_decode(src: &mut BytesMut) -> Option<LHMFrameHeader> {
        // Ensure we have the full required header bytes
        if src.len() < 8 {
            return None;
        }

        let id = src.get_u32();
        let length = src.get_u32();
        Some(LHMFrameHeader { length, id })
    }
}

#[derive(Default)]
pub struct LHMFrameCodec {
    header: Option<LHMFrameHeader>,
}

impl Encoder<LHMFrame> for LHMFrameCodec {
    type Error = std::io::Error;

    fn encode(&mut self, item: LHMFrame, dst: &mut BytesMut) -> Result<(), Self::Error> {
        dst.put_u32(item.id);
        dst.put_u32(item.body.len() as u32);
        dst.extend_from_slice(&item.body);
        Ok(())
    }
}

impl Decoder for LHMFrameCodec {
    type Item = LHMFrame;
    type Error = std::io::Error;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        let header = match self.header.as_mut() {
            Some(value) => value,
            None => match LHMFrameHeader::try_decode(src) {
                Some(value) => self.header.insert(value),
                None => return Ok(None),
            },
        };

        let length = header.length as usize;

        // Not enough bytes for the whole message
        if src.len() < length {
            return Ok(None);
        }

        let header = self
            .header
            .take()
            .expect("impossible to read a frame without a header");

        let bytes = src.split_to(length);

        Ok(Some(LHMFrame {
            id: header.id,
            body: bytes.freeze(),
        }))
    }
}
