use std::{collections::VecDeque, io::Seek};

use common::{
    demuxer::{Demuxer, DemuxingError},
    packet::Packet,
    time::{TimeBase, Timestamp},
    track::{BitstreamFormat, CodecId, TrackId, TrackInfo},
};

use crate::{
    cluster::Cluster,
    embl::{EBMLHeader, io::EBMLRead},
    info::Info,
    segment::Segment,
};

mod cluster;
mod embl;
mod info;
mod segment;
mod track;

#[derive(Debug)]
pub struct MatroskaDemuxer<T: EBMLRead + Seek> {
    reader: T,

    info: Info,
    timebase: TimeBase,
    tracks: Vec<TrackInfo>,

    clusters: Vec<Cluster>,
    cluster_index: Option<usize>,
    current_cluster_timecode: Timestamp,

    pending: VecDeque<Packet>,
}

impl<T: EBMLRead + Seek> MatroskaDemuxer<T> {
    pub fn new(mut reader: T) -> Result<Self, DemuxingError> {
        let header = reader.master_element::<EBMLHeader>(None)?;
        let segment = reader.master_element::<Segment>(None)?;

        if header.doc_type != "matroska" {
            return Err(DemuxingError::InvalidData(format!(
                "Invalid EBML doc type: expected 'matroska', got '{}'",
                header.doc_type
            )));
        }

        let timebase = TimeBase {
            num: segment.info.timecode_scale as u32,
            den: 1_000_000_000,
        };

        let mut tracks = Vec::new();
        for track in &segment.tracks.tracks {
            let track_info = TrackInfo {
                id: track.id(),
                kind: track.track_kind(),
                codec: track.codec_id(),
                timebase,
                extra_data: track.codec_private.clone(),
                bitstream_format: match track.codec_id() {
                    CodecId::H264 => BitstreamFormat::LengthPrefixed { nal_length_size: 4 },
                    _ => BitstreamFormat::Opaque,
                },
            };
            tracks.push(track_info);
        }

        Ok(Self {
            reader,

            info: segment.info,
            timebase,
            tracks,

            clusters: segment.clusters,
            cluster_index: None,
            current_cluster_timecode: Timestamp { value: 0, timebase },

            pending: VecDeque::with_capacity(128),
        })
    }

    fn fill_pending_packets(&mut self) -> Result<(), DemuxingError> {
        if self.cluster_index.is_none() {
            self.cluster_index = self
                .clusters
                .iter()
                .rposition(|cluster| cluster.timecode <= self.current_cluster_timecode.value)
                .or(Some(0));
        }

        if let Some(cluster_index) = self.cluster_index
            && cluster_index < self.clusters.len()
        {
            let cluster = &self.clusters[cluster_index];
            let packets = cluster.as_packets(&mut self.reader, self.timebase)?;
            self.pending.extend(packets);
            self.cluster_index = Some(cluster_index + 1);
        }

        Ok(())
    }
}

impl<T: EBMLRead + Seek> Demuxer for MatroskaDemuxer<T> {
    fn tracks(&self) -> &[TrackInfo] {
        &self.tracks
    }

    fn read_packet(&mut self) -> Result<Option<Packet>, DemuxingError> {
        loop {
            if let Some(packet) = self.pending.pop_front() {
                return Ok(Some(packet));
            }

            self.fill_pending_packets()?;

            if self.pending.is_empty() {
                return Ok(None);
            }
        }
    }

    fn seek(&mut self, track: TrackId, target: Timestamp) -> Result<(), DemuxingError> {
        self.pending.clear();
        self.cluster_index = None;
        self.current_cluster_timecode = target.rescale(self.timebase);

        let cluster_idx = self
            .clusters
            .iter()
            .enumerate()
            .rev()
            .find(|(_, c)| {
                c.timecode <= self.current_cluster_timecode.value
                    && c.blocks
                        .iter()
                        .any(|b| b.track_number() == track && b.is_keyframe())
            })
            .map(|(i, _)| i);

        self.cluster_index = cluster_idx.or(Some(0));

        Ok(())
    }

    fn duration(&self) -> Result<Timestamp, DemuxingError> {
        let duration = self.info.duration.ok_or(DemuxingError::InvalidData(
            "No duration in \\Segment\\Info".to_string(),
        ))?;

        Ok(Timestamp::new(duration as i64, self.timebase))
    }
}
