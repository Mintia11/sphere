use common::{
    packet::{Error, Frame, Packet, PacketDecoder},
    track::TrackInfo,
};

pub struct AacDecoder {}

impl PacketDecoder for AacDecoder {
    fn track(&mut self, track: &TrackInfo) -> Result<(), Error> {
        Ok(())
    }

    fn info_strings(&self) -> Vec<String> {
        vec![]
    }

    fn can_decode_track(&self) -> Result<bool, Error> {
        Ok(true)
    }

    fn start_decode_session(&mut self) -> Result<(), Error> {
        Ok(())
    }

    fn send_packet(&mut self, packet: Packet) -> Result<(), Error> {
        Ok(())
    }

    fn grab_frame(&self) -> Result<Option<Frame>, Error> {
        Ok(None)
    }
}
