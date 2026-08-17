use std::io::Seek;

use crate::{
    cluster::Cluster,
    embl::{
        EBMLMasterElement,
        io::{EBMLRead, Error},
    },
    info::Info,
    track::Tracks,
};

#[derive(Debug, Default)]
pub struct Segment {
    pub info: Info,
    pub tracks: Tracks,
    pub clusters: Vec<Cluster>,
}

impl<T: EBMLRead + Seek> EBMLMasterElement<T> for Segment {
    const ID: u32 = 0x18538067;

    fn visit_child(
        &mut self,
        sub_element: crate::embl::EBMLElement,
        reader: &mut T,
    ) -> Result<(), Error> {
        match sub_element.id {
            <Info as EBMLMasterElement<T>>::ID => {
                let info = reader.master_element::<Info>(Some(sub_element))?;
                self.info = info;
            }
            <Tracks as EBMLMasterElement<T>>::ID => {
                let tracks = reader.master_element::<Tracks>(Some(sub_element))?;
                self.tracks = tracks;
            }
            <Cluster as EBMLMasterElement<T>>::ID => {
                let cluster = reader.master_element::<Cluster>(Some(sub_element))?;
                self.clusters.push(cluster);
            }
            _ => {}
        }

        Ok(())
    }
}
