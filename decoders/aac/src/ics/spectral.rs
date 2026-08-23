use common::{bit_io::BitReader, byte_io::ByteRead, huffman::HuffmanTable, packet::Error};

use crate::{
    ics::{info::Info, scale_factors::ScaleFactors, section::SectionData},
    tables,
};

#[derive(Clone)]
pub struct SpectralData {
    pub coefficents: Box<[f32; 1024]>,
}

impl SpectralData {
    pub fn parse<T: ByteRead>(
        reader: &mut BitReader<T>,
        info: &Info,
        section: &SectionData,
        scale_factors: &ScaleFactors,
    ) -> Result<Self, Error> {
        let offsets = &info.sfb_offsets;

        let quad_tables = &tables::SPECTRUM_HUFFMAN_TABLES[0..4];
        let pair_tables = &tables::SPECTRUM_HUFFMAN_TABLES[4..10];
        let escape = &tables::SPECTRUM_HUFFMAN_TABLES[10];

        let mut coefficents = Box::new([0.0; 1024]);
        for (g, offsets) in offsets
            .iter()
            .enumerate()
            .take(info.window_group_length.len())
        {
            let group_start = info.window_group_length[..g].iter().sum::<u8>() as usize;
            let group_end = group_start + info.window_group_length[g] as usize;

            for sfb in 0..info.max_sfb {
                let start = offsets[sfb];
                let end = offsets[sfb + 1];

                let section = section.find_section(g, sfb).unwrap();
                let scale = scale_factors.groups[g][sfb];

                for w in group_start..group_end {
                    let dst = &mut coefficents[(start + w * 128)..(end + w * 128)];

                    match section.cb {
                        0 => {}
                        1 | 2 => {
                            for out in dst.chunks_exact_mut(4) {
                                let quad = Self::decode_quads_signed(
                                    reader,
                                    &quad_tables[section.cb as usize - 1],
                                    scale,
                                )?;
                                out.copy_from_slice(&quad);
                            }
                        }
                        3 | 4 => {
                            for out in dst.chunks_exact_mut(4) {
                                let quad = Self::decode_quads_unsigned(
                                    reader,
                                    &quad_tables[section.cb as usize - 1],
                                    scale,
                                )?;
                                out.copy_from_slice(&quad);
                            }
                        }
                        5 | 6 => {
                            for out in dst.chunks_exact_mut(2) {
                                let (a, b) = Self::decode_pairs_signed::<_, 9>(
                                    reader,
                                    &pair_tables[section.cb as usize - 5],
                                    scale,
                                )?;
                                out[0] = a;
                                out[1] = b;
                            }
                        }
                        7 | 8 => {
                            for out in dst.chunks_exact_mut(2) {
                                let (a, b) = Self::decode_pairs_unsigned::<_, 8>(
                                    reader,
                                    &pair_tables[section.cb as usize - 5],
                                    scale,
                                )?;
                                out[0] = a;
                                out[1] = b;
                            }
                        }
                        9 | 10 => {
                            for out in dst.chunks_exact_mut(2) {
                                let (a, b) = Self::decode_pairs_unsigned::<_, 13>(
                                    reader,
                                    &pair_tables[section.cb as usize - 5],
                                    scale,
                                )?;
                                out[0] = a;
                                out[1] = b;
                            }
                        }
                        11 => {
                            for out in dst.chunks_exact_mut(2) {
                                let (a, b) = Self::decode_pairs_escape(reader, escape, scale)?;
                                out[0] = a;
                                out[1] = b;
                            }
                        }
                        13 => {
                            let mut state = 0x1F2E3D4Cu32;
                            let mut lcg = || -> i32 {
                                state = state.wrapping_mul(1664525).wrapping_add(1013904223);
                                state as i32
                            };

                            let mut energy = 0.0;
                            for out in dst.iter_mut() {
                                *out = (lcg() >> 16) as f32;
                                energy += *out * *out;
                            }

                            let scale_factor = scale / energy.sqrt();
                            for out in dst.iter_mut() {
                                *out *= scale_factor;
                            }
                        }
                        14 | 15 => {}
                        cb => todo!("spectral data decoding for codebook {cb}"),
                    }
                }
            }
        }

        Ok(SpectralData { coefficents })
    }

    fn decode_sign<T: ByteRead>(reader: &mut BitReader<T>) -> Result<f32, Error> {
        let sign = if reader.read_bit()? { -1.0 } else { 1.0 };
        Ok(sign)
    }

    fn decode_quads_unsigned<T: ByteRead>(
        reader: &mut BitReader<T>,
        table: &HuffmanTable,
        scale: f32,
    ) -> Result<[f32; 4], Error> {
        let val = table.decode(reader)?;
        fn index_to_quad(mut idx: u16) -> (u8, u8, u8, u8) {
            let d = (idx % 3) as u8;
            idx /= 3;
            let c = (idx % 3) as u8;
            idx /= 3;
            let b = (idx % 3) as u8;
            idx /= 3;
            let a = (idx % 3) as u8;
            (a, b, c, d)
        }

        let (a, b, c, d) = index_to_quad(val);
        let table = [0.0, scale, 2.5198421 * scale];

        let mut out = [0.0; 4];
        if a != 0 {
            out[0] = Self::decode_sign(reader)? * table[a as usize];
        }
        if b != 0 {
            out[1] = Self::decode_sign(reader)? * table[b as usize];
        }
        if c != 0 {
            out[2] = Self::decode_sign(reader)? * table[c as usize];
        }
        if d != 0 {
            out[3] = Self::decode_sign(reader)? * table[d as usize];
        }

        Ok(out)
    }

    fn decode_quads_signed<T: ByteRead>(
        reader: &mut BitReader<T>,
        table: &HuffmanTable,
        scale: f32,
    ) -> Result<[f32; 4], Error> {
        let val = table.decode(reader)?;
        fn index_to_quad(mut idx: u16) -> (u8, u8, u8, u8) {
            let d = (idx % 3) as u8;
            idx /= 3;
            let c = (idx % 3) as u8;
            idx /= 3;
            let b = (idx % 3) as u8;
            idx /= 3;
            let a = (idx % 3) as u8;
            (a, b, c, d)
        }

        let (a, b, c, d) = index_to_quad(val);
        let table = [-scale, 0.0, scale];

        Ok([
            table[a as usize],
            table[b as usize],
            table[c as usize],
            table[d as usize],
        ])
    }

    fn decode_pairs_signed<T: ByteRead, const MOD: u16>(
        reader: &mut BitReader<T>,
        table: &HuffmanTable,
        scale: f32,
    ) -> Result<(f32, f32), Error> {
        let val = table.decode(reader)?;
        let half = (MOD / 2) as i16;
        let (a, b) = ((val / MOD) as i16 - half, (val % MOD) as i16 - half);
        let (a, b) = (
            a.signum() as f32 * (a.unsigned_abs() as f32).powf(4.0 / 3.0),
            b.signum() as f32 * (b.unsigned_abs() as f32).powf(4.0 / 3.0),
        );
        Ok((a * scale, b * scale))
    }

    fn decode_pairs_unsigned<T: ByteRead, const MOD: u16>(
        reader: &mut BitReader<T>,
        table: &HuffmanTable,
        scale: f32,
    ) -> Result<(f32, f32), Error> {
        let val = table.decode(reader)?;
        let (a, b) = (val / MOD, val % MOD);
        let (a, b) = ((a as f32).powf(4.0 / 3.0), (b as f32).powf(4.0 / 3.0));

        let sign_a = if a != 0.0 {
            Self::decode_sign(reader)?
        } else {
            1.0
        };
        let sign_b = if b != 0.0 {
            Self::decode_sign(reader)?
        } else {
            1.0
        };

        Ok((a * sign_a * scale, b * sign_b * scale))
    }

    fn decode_pairs_escape<T: ByteRead>(
        reader: &mut BitReader<T>,
        table: &HuffmanTable,
        scale: f32,
    ) -> Result<(f32, f32), Error> {
        let val = table.decode(reader)?;
        let (a, b) = (val / 17, val % 17);

        let sign_a = if a != 0 {
            Self::decode_sign(reader)?
        } else {
            1.0
        };
        let sign_b = if b != 0 {
            Self::decode_sign(reader)?
        } else {
            1.0
        };

        let a = if a == 16 {
            Self::decode_escape(reader)?
        } else {
            a as usize
        };
        let b = if b == 16 {
            Self::decode_escape(reader)?
        } else {
            b as usize
        };

        let a = (a as f32).powf(4.0 / 3.0);
        let b = (b as f32).powf(4.0 / 3.0);

        Ok((a * sign_a * scale, b * sign_b * scale))
    }

    fn decode_escape<T: ByteRead>(reader: &mut BitReader<T>) -> Result<usize, Error> {
        let mut n = 0;
        while reader.read_bit()? {
            n += 1;
        }
        assert!(n < 9, "escape value is too large: {}", n);
        let value = reader.read_bits(n + 4)? as usize + (1 << (n + 4));
        Ok(value)
    }
}
