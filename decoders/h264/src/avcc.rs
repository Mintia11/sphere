use common::{bit_io::BitReader, byte_io::ByteRead, packet::Error};

pub struct Avcc {
    profile: Profile,
    level: Level,
}

impl Avcc {
    pub fn parse<T: ByteRead>(reader: &mut BitReader<T>) -> Result<Self, Error> {
        let version = reader.read_bits(8)?;
        if version != 1 {
            return Err(Error::InvalidData(
                "avcc: configurationVersion should always be 1".to_string(),
            ));
        }

        let profile = reader.read_bits(8)?;
        let profile_compatibility = reader.read_bits(8)?;
        let level = reader.read_bits(8)?;
        let _ = reader.read_bits(6)?;
        let lenght_size = reader.read_bits(2)? + 1;
        let _ = reader.read_bits(3)?;
        let sps_num = reader.read_bits(5)?;

        Ok(Self {
            profile: Profile::parse(profile as u8, profile_compatibility as u8),
            level: Level::parse(level as u8),
        })
    }
}

#[derive(Debug)]
pub enum Profile {
    Baseline,
    ConstrainedBaseline,
}

impl Profile {
    pub fn parse(profile: u8, profile_compatibility: u8) -> Profile {
        match profile {
            _ => todo!("unknown profile: {profile}"),
        }
    }
}

#[derive(Debug)]
pub enum Level {}

impl Level {
    pub fn parse(level: u8) -> Level {
        match level {
            _ => todo!("unknown level: {level}"),
        }
    }
}
