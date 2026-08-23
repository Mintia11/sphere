use std::fmt::Display;

use common::{bit_io::BitReader, byte_io::ByteReader, packet::Error};
use derive_try_from_primitive::TryFromPrimitive;

#[derive(Debug, Clone, Copy)]
pub struct Config {
    pub object_type: AudioObjectType,
    pub sampling_frequency: u32,
    pub channel_configuration: ChannelConfiguration,
}

impl Config {
    pub fn parse(codec_private_data: &[u8]) -> Result<Self, Error> {
        let reader = ByteReader::new(codec_private_data);
        let mut reader = BitReader::new(reader);

        let audio_object_type = reader.read_bits(5)?;
        let audio_object_type = if audio_object_type == 31 {
            let audio_object_type_ext = reader.read_bits(6)?;
            32 + audio_object_type_ext
        } else {
            audio_object_type
        };

        let audio_object_type: AudioObjectType = (audio_object_type as u8)
            .try_into()
            .map_err(|_| Error::InvalidData("unknown audio object type".to_string()))?;

        let sampling_frequency_index = reader.read_bits(4)?;
        let sampling_frequency = if sampling_frequency_index == 0x0F {
            reader.read_bits(24)? as u32
        } else {
            const SAMPLING_FREQUENCIES: [u32; 16] = [
                96000, 88200, 64000, 48000, 44100, 32000, 24000, 22050, 16000, 12000, 11025, 8000,
                7350, 0, 0, 0,
            ];
            SAMPLING_FREQUENCIES[sampling_frequency_index as usize]
        };

        let channel_configuration: ChannelConfiguration =
            (reader.read_bits(4)? as u8)
                .try_into()
                .map_err(|_| Error::InvalidData("invalid channel configuration".to_string()))?;

        let frame_length_flag = reader.read_bit()?;
        let depends_on_core_coder = reader.read_bit()?;
        let core_coder_delay = if depends_on_core_coder {
            Some(reader.read_bits(12)?)
        } else {
            None
        };
        let extension_flag = reader.read_bit()?;

        if frame_length_flag {
            todo!("Handle frame length flag");
        }

        if core_coder_delay.is_some() {
            todo!("Handle core coder delay");
        }

        if extension_flag {
            todo!("Handle extension flag");
        }

        Ok(Config {
            object_type: audio_object_type,
            sampling_frequency,
            channel_configuration,
        })
    }
}

#[derive(Debug, TryFromPrimitive, PartialEq, Eq, Clone, Copy)]
#[repr(u8)]
pub enum ChannelConfiguration {
    Escape,
    Mono,
    Stereo,
    ThreeChannels,
    FourChannels,
    FiveChannels,
    FivePointOneChannels,
    SevenPointOneChannels,
}

impl ChannelConfiguration {
    pub fn channel_count(&self) -> usize {
        match self {
            ChannelConfiguration::Escape => 0,
            ChannelConfiguration::Mono => 1,
            ChannelConfiguration::Stereo => 2,
            ChannelConfiguration::ThreeChannels => 3,
            ChannelConfiguration::FourChannels => 4,
            ChannelConfiguration::FiveChannels => 5,
            ChannelConfiguration::FivePointOneChannels => 6,
            ChannelConfiguration::SevenPointOneChannels => 8,
        }
    }
}

impl Display for ChannelConfiguration {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ChannelConfiguration::Escape => write!(f, "escape"),
            ChannelConfiguration::Mono => write!(f, "mono"),
            ChannelConfiguration::Stereo => write!(f, "2.0"),
            ChannelConfiguration::ThreeChannels => write!(f, "3.0"),
            ChannelConfiguration::FourChannels => write!(f, "4.0"),
            ChannelConfiguration::FiveChannels => write!(f, "5.0"),
            ChannelConfiguration::FivePointOneChannels => write!(f, "5.1"),
            ChannelConfiguration::SevenPointOneChannels => write!(f, "7.1"),
        }
    }
}

#[derive(Debug, TryFromPrimitive, PartialEq, Eq, Clone, Copy)]
#[repr(u8)]
pub enum AudioObjectType {
    AacLc = 2,
}
