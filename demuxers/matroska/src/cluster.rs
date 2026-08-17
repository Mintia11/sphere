use std::io::Seek;

use common::{
    demuxer::DemuxingError,
    packet::{Packet, PacketFlags},
    time::{TimeBase, Timestamp},
    track::TrackId,
};

use crate::embl::{
    EBMLElement, EBMLMasterElement,
    io::{EBMLRead, Error},
};

#[derive(Debug, Default)]
pub struct Cluster {
    pub timecode: i64,
    pub blocks: Vec<Block>,
}

impl Cluster {
    pub fn as_packets<T: EBMLRead + Seek>(
        &self,
        mut reader: T,
        timebase: TimeBase,
    ) -> Result<Vec<Packet>, DemuxingError> {
        let cluster_timestamp = Timestamp {
            value: self.timecode,
            timebase,
        };

        let mut packets = Vec::new();
        for block in &self.blocks {
            let mut block_packets = block.into_packet(&mut reader, cluster_timestamp)?;
            packets.append(&mut block_packets);
        }

        Ok(packets)
    }
}

impl<T: EBMLRead + Seek> EBMLMasterElement<T> for Cluster {
    const ID: u32 = 0x1F43B675;

    fn visit_child(&mut self, sub_element: EBMLElement, reader: &mut T) -> Result<(), Error> {
        match sub_element.id {
            0xBF => {} // I don't know what is this
            0xE7 => self.timecode = reader.uinteger(sub_element.data_size())? as i64,
            0xA0 => {} // TODO: handle BlockGroup
            0xA3 => {
                let block = Block::SimpleBlock {
                    track_number: reader.vint()?,
                    timecode: reader.be_u16()? as i16,
                    flags: reader.be_u8()?,
                    element: EBMLElement {
                        data_start: reader.stream_position()?,
                        ..sub_element
                    },
                };

                self.blocks.push(block);
            }
            _ => {}
        }

        Ok(())
    }

    fn validate(&self) -> Result<(), Error> {
        if self.blocks.is_empty() {
            return Err(Error::InvalidData(
                "Cluster must contain at least one block".to_string(),
            ));
        }

        Ok(())
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Block {
    SimpleBlock {
        track_number: u64,
        timecode: i16,
        flags: u8,
        element: EBMLElement,
    },
}

impl Block {
    pub fn into_packet<T: EBMLRead + Seek>(
        self,
        reader: T,
        cluster_timestamp: Timestamp,
    ) -> Result<Vec<Packet>, DemuxingError> {
        match self {
            Block::SimpleBlock {
                track_number,
                timecode,
                flags,
                element,
            } => {
                if flags & 0x6 != 0 {
                    return Err(DemuxingError::InvalidData(
                        "Unsupported SimpleBlock lacing".to_string(),
                    ));
                }

                let timestamp = Timestamp {
                    value: cluster_timestamp.value + timecode as i64,
                    timebase: cluster_timestamp.timebase,
                };

                let is_keyframe = flags & 0x80 != 0;
                let is_disposable = flags & 0x1 != 0;

                let data = element.data(reader).map_err(|e| match e {
                    Error::Io(e) => DemuxingError::Io(e),
                    _ => unreachable!("shouldn't get any other error other than I/O"),
                })?;

                let packet = Packet {
                    track: track_number as TrackId,
                    pts: timestamp,
                    dts: timestamp,
                    duration: None,
                    data: data.into(),
                    flags: if is_keyframe {
                        PacketFlags::KEYFRAME
                    } else {
                        PacketFlags::empty()
                    } | if is_disposable {
                        PacketFlags::DISCARD
                    } else {
                        PacketFlags::empty()
                    },
                };

                Ok(vec![packet])
            }
        }
    }

    pub fn track_number(&self) -> TrackId {
        match self {
            Block::SimpleBlock { track_number, .. } => *track_number as TrackId,
        }
    }

    pub fn is_keyframe(&self) -> bool {
        match self {
            Block::SimpleBlock { flags, .. } => flags & 0x80 != 0,
        }
    }
}
