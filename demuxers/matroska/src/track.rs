use std::io::Seek;

use bytes::Bytes;
use common::track::{CodecId, TrackId, TrackKind};

use crate::embl::{
    EBMLElement, EBMLMasterElement,
    io::{EBMLRead, Error},
};

#[derive(Debug, Default)]
pub struct Tracks {
    pub tracks: Vec<Track>,
}

impl<T: EBMLRead + Seek> EBMLMasterElement<T> for Tracks {
    const ID: u32 = 0x1654AE6B;

    fn visit_child(&mut self, sub_element: EBMLElement, reader: &mut T) -> Result<(), Error> {
        match sub_element.id {
            <Track as EBMLMasterElement<T>>::ID => {
                let track = reader.master_element::<Track>(Some(sub_element))?;
                self.tracks.push(track);
            }
            _ => {}
        }

        Ok(())
    }
}

#[derive(Debug, Default)]
pub struct Track {
    pub number: u64,
    pub uid: u64,
    pub track_type: u64,
    pub codec_id: String,
    pub codec_private: Option<Bytes>,
}

impl Track {
    pub fn id(&self) -> TrackId {
        self.number as TrackId
    }

    pub fn track_kind(&self) -> TrackKind {
        match self.track_type {
            1 => TrackKind::Video,
            2 => TrackKind::Audio,
            17 => TrackKind::Subtitle,
            _ => TrackKind::Unknown,
        }
    }

    pub fn codec_id(&self) -> CodecId {
        match self.codec_id.as_str() {
            "V_MPEG4/ISO/AVC" => CodecId::H264,
            "A_AAC" => CodecId::Aac,
            x => CodecId::Unknown(x.to_string()),
        }
    }
}

impl<T: EBMLRead + Seek> EBMLMasterElement<T> for Track {
    const ID: u32 = 0xAE;

    fn visit_child(&mut self, sub_element: EBMLElement, reader: &mut T) -> Result<(), Error> {
        match sub_element.id {
            0xD7 => self.number = reader.uinteger(sub_element.data_size())?,
            0x73C5 => self.uid = reader.uinteger(sub_element.data_size())?,
            0x83 => self.track_type = reader.uinteger(sub_element.data_size())?,
            0x86 => self.codec_id = reader.string(sub_element.data_size())?,
            0x63A2 => self.codec_private = Some(reader.binary(sub_element.data_size())?.into()),
            _ => {}
        }

        Ok(())
    }
}
