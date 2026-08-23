use common::{bit_io::BitReader, byte_io::ByteRead, packet::Error};

use crate::ics::info::Info;

#[derive(Clone)]
pub struct PulseData {
    pub start_sfb: usize,
    pub pulses: Vec<Pulse>,
}

#[derive(Clone, Copy)]
pub struct Pulse {
    pub offset: usize,
    pub amp: u8,
}

impl PulseData {
    pub fn parse<T: ByteRead>(
        reader: &mut BitReader<T>,
        info: &Info,
    ) -> Result<Option<Self>, Error> {
        let pulse_data_present = reader.read_bit()?;
        if !pulse_data_present {
            return Ok(None);
        }

        let start_sfb = reader.read_bits(6)? as usize;
        let num_pulses = reader.read_bits(2)? as usize + 1;

        if start_sfb + num_pulses > info.max_sfb {
            panic!("pulse data exceeds max_sfb");
        }

        let mut pulses = Vec::with_capacity(num_pulses);
        for _ in 0..num_pulses {
            let offset = reader.read_bits(5)? as usize;
            let amp = reader.read_bits(4)? as u8;
            pulses.push(Pulse { offset, amp });
        }

        Ok(Some(PulseData { start_sfb, pulses }))
    }
}
