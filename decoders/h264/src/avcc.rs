use std::{
    fmt::Debug,
    io::{Seek, SeekFrom},
};

use bytes::Bytes;
use common::{
    bit_io::BitReader,
    byte_io::{ByteRead, ByteReader},
    packet::Error,
};
use derive_try_from_primitive::TryFromPrimitive;
use etna::vk;

use crate::nal::RawNal;

#[derive(Debug)]
pub struct Avcc {
    pub sps: Vec<RawNal>,
    pub pps: Vec<RawNal>,
}

impl Avcc {
    pub fn parse(bytes: &Bytes) -> Result<Self, Error> {
        let reader = ByteReader::new(bytes);
        let mut reader = BitReader::new(reader);

        let version = reader.read_bits(8)?;
        if version != 1 {
            return Err(Error::InvalidData(
                "avcc: configurationVersion should always be 1".to_string(),
            ));
        }

        let _profile = reader.read_bits(8)?;
        let _profile_compatibility = reader.read_bits(8)?;
        let _level = reader.read_bits(8)?;
        let _ = reader.read_bits(6)?;
        let _lenght_size = reader.read_bits(2)? + 1;
        let _ = reader.read_bits(3)?;
        let sps_num = reader.read_bits(5)?;
        reader.byte_align();

        let byte_reader = reader.inner_mut();
        byte_reader.seek(SeekFrom::Start(6))?;

        let mut sps = Vec::new();
        for _ in 0..sps_num {
            let lenght = byte_reader.be_u16()? as usize;
            let current_pos = byte_reader.stream_position()? as usize;
            let bytes = byte_reader
                .inner()
                .slice(current_pos..(current_pos + lenght));
            byte_reader.seek(SeekFrom::Current(lenght as i64))?;

            sps.push(RawNal::parse(&bytes)?);
        }

        let pps_num = byte_reader.be_u8()?;
        let mut pps = Vec::new();
        for _ in 0..pps_num {
            let lenght = byte_reader.be_u16()? as usize;
            let current_pos = byte_reader.stream_position()? as usize;
            let bytes = byte_reader
                .inner()
                .slice(current_pos..(current_pos + lenght));
            byte_reader.seek(SeekFrom::Current(lenght as i64))?;

            pps.push(RawNal::parse(&bytes)?);
        }

        Ok(Self { sps, pps })
    }
}

#[derive(Debug, Clone, Copy)]
#[repr(u32)]
pub enum Profile {
    High = 100,
}

impl Profile {
    pub fn parse(profile: u8, _profile_compatibility: u8) -> Profile {
        match profile {
            100 => Profile::High,
            _ => todo!("unknown profile: {profile}"),
        }
    }

    pub fn is_high(&self) -> bool {
        matches!(self, Profile::High)
    }
}

impl From<Profile> for vk::native::StdVideoH264ProfileIdc {
    fn from(value: Profile) -> Self {
        value as vk::native::StdVideoH264ProfileIdc
    }
}

#[derive(Clone, Copy)]
pub struct Level(u8, u8);

impl Level {
    pub fn parse(level: u8) -> Level {
        Level(level / 10, level % 10)
    }
}

impl Debug for Level {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}.{}", self.0, self.1)
    }
}

impl From<Level> for vk::native::StdVideoH264LevelIdc {
    fn from(value: Level) -> Self {
        (value.0 * 10 + value.1) as u32
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, TryFromPrimitive)]
#[repr(u64)]
pub enum ChromaFormat {
    Monochrome,
    Yuv420,
    Yuv422,
    Yuv444,
}

impl From<ChromaFormat> for vk::VideoChromaSubsamplingFlagsKHR {
    fn from(value: ChromaFormat) -> Self {
        match value {
            ChromaFormat::Monochrome => vk::VideoChromaSubsamplingFlagsKHR::MONOCHROME,
            ChromaFormat::Yuv420 => vk::VideoChromaSubsamplingFlagsKHR::TYPE_420,
            ChromaFormat::Yuv422 => vk::VideoChromaSubsamplingFlagsKHR::TYPE_422,
            ChromaFormat::Yuv444 => vk::VideoChromaSubsamplingFlagsKHR::TYPE_444,
        }
    }
}
