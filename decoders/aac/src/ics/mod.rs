use common::{bit_io::BitReader, byte_io::ByteRead, packet::Error};

use crate::{
    config::Config,
    ics::{
        gain_control::GainControl, info::Info, pulse::PulseData, scale_factors::ScaleFactors,
        section::SectionData, spectral::SpectralData, tns::TNSData,
    },
};
pub use info::WindowSequence;

mod gain_control;
pub(crate) mod info;
mod pulse;
mod scale_factors;
mod section;
mod spectral;
mod tns;

#[derive(Clone)]
pub struct Ics {
    pub info: Info,
    pub pulse: Option<PulseData>,
    pub tns: Option<TNSData>,
    pub gain_control: Option<GainControl>,
    pub spectral: SpectralData,
}

impl Ics {
    pub fn parse<T: ByteRead>(
        reader: &mut BitReader<T>,
        config: &Config,
        info: Option<Info>,
    ) -> Result<Self, Error> {
        let global_gain = reader.read_bits(8)? as u8;
        let info = match info {
            Some(info) => info,
            None => Info::parse(reader, config)?,
        };
        let section = SectionData::parse(reader, &info)?;
        let scale_factors = ScaleFactors::parse(reader, global_gain, &info, &section)?;
        let pulse = PulseData::parse(reader, &info)?;
        assert!(pulse.is_none(), "todo: use pulse data");
        let tns = TNSData::parse(reader)?;
        assert!(tns.is_none(), "todo: use tns data");
        let gain_control = GainControl::parse(reader)?;
        assert!(gain_control.is_none(), "todo: use gain control");
        let spectral = SpectralData::parse(reader, &info, &section, &scale_factors)?;

        Ok(Self {
            info,
            pulse,
            tns,
            gain_control,
            spectral,
        })
    }
}
