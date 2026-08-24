use std::f32::consts;

use common::{bit_io::BitReader, byte_io::ByteRead, packet::Error};

use crate::ics::{WindowSequence, info::Info};

#[derive(Clone)]
pub struct TNSData {
    pub windows: Vec<(u8, Vec<TnsFilter>)>,
}

#[derive(Clone)]
pub struct TnsFilter {
    pub length: usize,
    pub order: usize,
    pub direction: bool,
    pub coef_compress: bool,
    pub coefs: [f32; 21],
}

impl TNSData {
    pub fn parse<T: ByteRead>(
        reader: &mut BitReader<T>,
        info: &Info,
    ) -> Result<Option<Self>, Error> {
        let tns_data_present = reader.read_bit()?;
        if !tns_data_present {
            return Ok(None);
        }

        let is_short = info.window_sequence == WindowSequence::EightShort;
        let num_windows = if is_short { 8 } else { 1 };

        let mut windows = Vec::with_capacity(num_windows);

        for _ in 0..num_windows {
            let num_filters = reader.read_bits(if is_short { 1 } else { 2 })? as usize;

            if num_filters == 0 {
                windows.push((0, Vec::new()));
                continue;
            }

            let coef_res = reader.read_bit()?;
            let mut filters = Vec::with_capacity(num_filters);

            for _ in 0..num_filters {
                let length = reader.read_bits(if is_short { 4 } else { 6 })? as usize;
                let order = reader.read_bits(if is_short { 3 } else { 5 })? as usize;

                let mut filter = TnsFilter {
                    length,
                    order,
                    direction: false,
                    coef_compress: false,
                    coefs: [0.0; 21],
                };

                if order > 0 {
                    filter.direction = reader.read_bit()?;
                    filter.coef_compress = reader.read_bit()?;

                    let coef_bits = coef_res as u32 + 3 - filter.coef_compress as u32;

                    fn sign_extend(value: u32, bits: u32) -> i8 {
                        let shift = 32 - bits;
                        ((value << shift) as i32 >> shift) as i8
                    }

                    let fac_base = if coef_res { 8.0 } else { 4.0 };

                    let iqfac = (fac_base - 0.5) / consts::FRAC_PI_2;
                    let iqfac_m = (fac_base + 0.5) / consts::FRAC_PI_2;

                    let mut coefs = [0.0; 20];
                    for out in coefs.iter_mut().take(order) {
                        let coef =
                            sign_extend(reader.read_bits(coef_bits)? as u32, coef_bits) as f32;
                        let coef = if coef >= 0.0 {
                            coef / iqfac
                        } else {
                            coef / iqfac_m
                        };

                        *out = coef;
                    }

                    let mut buf = vec![0.0];
                    for i in 1..=order {
                        for j in 1..i {
                            buf.push(filter.coefs[j - 1] + coefs[i - 1] * filter.coefs[i - j - 1]);
                        }

                        filter.coefs[..(i - 1)].copy_from_slice(&buf[1..i]);
                        filter.coefs[i - 1] = coefs[i - 1];
                    }
                }

                filters.push(filter);
            }

            windows.push((coef_res as u8, filters));
        }

        Ok(Some(TNSData { windows }))
    }
}
