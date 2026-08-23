use common::{
    bit_io::BitReader,
    byte_io::ByteReader,
    packet::{Error, Frame, Packet, PacketDecoder},
    track::TrackInfo,
};

use crate::{bitstream::SyntaxElement, config::Config};

mod bitstream;
mod config;
mod ics;
mod tables;

#[derive(Default)]
pub struct AacDecoder {
    config: Option<Config>,
}

impl PacketDecoder for AacDecoder {
    fn track(&mut self, track: &TrackInfo) -> Result<(), Error> {
        let private_data = track.extra_data.as_ref().ok_or(Error::InvalidData(
            "Track has no codec private data".to_string(),
        ))?;

        self.config = Some(Config::parse(private_data)?);

        Ok(())
    }

    fn info_strings(&self) -> Vec<String> {
        let mut info_strings = Vec::new();

        if let Some(config) = self.config {
            info_strings.push(format!("Object Type: {:?}", config.object_type));
            info_strings.push(format!("Sample Rate: {} Hz", config.sampling_frequency));
            info_strings.push(format!(
                "Channel Configuration: {}",
                config.channel_configuration
            ));
        }

        info_strings
    }

    fn can_decode_track(&self) -> Result<bool, Error> {
        Ok(self.config.is_some())
    }

    fn start_decode_session(&mut self) -> Result<(), Error> {
        Ok(())
    }

    fn send_packet(&mut self, packet: Packet) -> Result<(), Error> {
        let reader = ByteReader::new(packet.data);
        let mut reader = BitReader::new(reader);

        let syntax_elements = SyntaxElement::parse_all(&mut reader, self.config.as_ref().unwrap())?;

        Ok(())
    }

    fn grab_frame(&self) -> Result<Option<Frame>, Error> {
        Ok(None)
    }
}
