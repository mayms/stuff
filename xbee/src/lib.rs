#![no_std]

pub enum ApiIdentifier {
    /** 64-bit Transmit Request */
    TxReq
}

impl ApiIdentifier {
    fn value(&self) -> u8 {
        match self {
            ApiIdentifier::TxReq => 0x00
        }
    }
}

pub struct Packet {
    bytes: [u8; 23],
    length: usize,
}

const DST_OFFSET: usize = 5;
const DATA_OFFSET: usize = 14;

impl Packet {
    pub fn new(api_identifier: ApiIdentifier, dst: [u8; 8], data: &[u8]) -> Packet {
        let mut packet = Packet {
            bytes: [0x00; 23],
            length: DATA_OFFSET + data.len() + 1,
        };
        packet.bytes[0] = 0x7e;
        let i1 = data.len() + 3 + dst.len();
        packet.bytes[1] = (i1 & 0xff00) as u8;
        packet.bytes[2] = (i1 & 0x00ff) as u8;
        packet.bytes[3] = api_identifier.value();
        packet.bytes[4] = 0x00; // api_frame_id
        dst.iter().enumerate().for_each(|(i, e)| packet.bytes[DST_OFFSET + i] = *e);
        packet.bytes[13] = 0x00; // options
        data.iter().enumerate().for_each(|(i, e)| packet.bytes[DATA_OFFSET + i] = *e);
        packet.bytes[DATA_OFFSET + data.len()] = packet.compute_checksum();
        return packet;
    }

    fn compute_checksum(&self) -> u8 {
        let sum_u16 = *self.api_identifier() as u16 +
            *self.api_frame_id() as u16 +
            self.destination_address().iter().map(|x| *x as u16).fold(0, |acc, x| acc + x) +
            *self.options() as u16 +
            self.data().iter().map(|x| *x as u16).fold(0, |acc, x| acc + x);
        return 0xFF - (sum_u16 & 0x00FF) as u8;
    }

    fn api_identifier(&self) -> &u8 {
        &self.bytes[3]
    }

    fn api_frame_id(&self) -> &u8 {
        &self.bytes[4]
    }

    fn destination_address(&self) -> &[u8] {
        &self.bytes.as_slice() //
            .split_at(DST_OFFSET).1 // strip prefix
            .split_at(8).0 // strip postfix
    }

    fn options(&self) -> &u8 {
        &self.bytes[13]
    }

    fn data(&self) -> &[u8] {
        &self.bytes.as_slice() //
            .split_at(DATA_OFFSET).1 // strip prefix
            .split_at(self.length - DATA_OFFSET - 1).0 // strip postfix
    }

    pub fn iter(&self) -> PacketIterator {
        return PacketIterator {
            packet: self,
            index: 0,
        };
    }
}

pub struct PacketIterator<'a> {
    packet: &'a Packet,
    index: usize,
}

impl Iterator for PacketIterator<'_> {
    type Item = u8;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index < self.packet.length.into() {
            let byte = self.packet.bytes[self.index];
            self.index += 1;
            return Some(byte);
        }
        return None;
    }
}

#[cfg(test)]
mod tests {
    use crate::ApiIdentifier::TxReq;
    use super::*;

    #[test]
    fn new_packet() {
        let actual = Packet::new(TxReq,
                                 [0x00, 0x13, 0xA2, 0x00, 0x40, 0x64, 0x03, 0x75],
                                 &[0xff, 0x41, 0x42, 0x43, 0x44, 0x45, 0x46, 0xff]);
        assert_eq!(actual.bytes, [ // taken from XCTU
            0x7e, // start
            0x00, 0x13, // len
            0x00, // api_identifier
            0x00, // api_frame_id
            0x00, 0x13, 0xA2, 0x00, 0x40, 0x64, 0x03, 0x75, // dst
            0x00, // options
            0xff, 0x41, 0x42, 0x43, // data
            0x44, 0x45, 0x46, 0xff, // data
            0x9b, // checksum
        ]);
    }

    #[test]
    fn data() {
        let expected_data = [0x00, 0x0f, 0xf0, 0xff];
        let actual = Packet::new(TxReq,
                                 [0x00, 0x13, 0xA2, 0x00, 0x40, 0x64, 0x73, 0x46],
                                 &expected_data);
        let mut max_i = 0;
        for (i, byte) in actual.data().iter().enumerate() {
            let expected_byte = expected_data[i];
            max_i = i;
            assert_eq!(expected_byte, *byte);
        }
        assert_eq!(4, max_i + 1);
    }

    #[test]
    fn destination_address() {
        let expected_destination_address = [0x00, 0x13, 0xA2, 0x00, 0x40, 0x64, 0x73, 0x46];
        let actual = Packet::new(TxReq,
                                 expected_destination_address,
                                 &[0x00, 0x0f, 0xf0, 0xff]);
        let mut max_i = 0;
        for (i, byte) in actual.destination_address().iter().enumerate() {
            let expected_byte = expected_destination_address[i];
            max_i = i;
            assert_eq!(expected_byte, *byte);
        }
        assert_eq!(8, max_i + 1);
    }

    #[test]
    fn iter() {
        let actual = Packet::new(TxReq,
                                 [0x00, 0x13, 0xA2, 0x00, 0x40, 0x64, 0x73, 0x46],
                                 &[0x00, 0x0f, 0xf0, 0xff]);
        let mut max_i = 0;
        for (i, byte) in actual.iter().enumerate() {
            let expected_byte = actual.bytes[i];
            max_i = i;
            assert_eq!(expected_byte, byte);
        }
        assert_eq!(actual.length, max_i + 1);
    }
}
