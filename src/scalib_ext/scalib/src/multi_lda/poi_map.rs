use itertools::Itertools;
use serde::{Deserialize, Serialize};

use crate::{Result, ScalibError};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PoiMap {
    new2old: Vec<u32>,
    old2new: Vec<Option<u32>>,
}

impl PoiMap {
    pub fn new(ns: u32, pois: impl Iterator<Item = u32>) -> Result<Self> {
        let mut used_pois = vec![false; ns as usize];
        for poi in pois {
            *used_pois
                .get_mut(poi as usize)
                .ok_or(ScalibError::PoiOutOfBound)? = true;
        }
        let new2old = used_pois
            .iter()
            .positions(|x| *x)
            .map(|x| x as u32)
            .collect_vec();
        let mut cnt = 0;
        let old2new = used_pois
            .iter()
            .map(|x| {
                x.then(|| {
                    cnt += 1;
                    cnt - 1
                })
            })
            .collect_vec();

        Ok(Self { new2old, old2new })
    }
    pub fn len(&self) -> u32 {
        self.new2old.len() as u32
    }
    pub fn to_new(&self, new: u32) -> Option<u32> {
        self.old2new[new as usize]
    }
    pub fn kept_indices(&self) -> &[u32] {
        self.new2old.as_slice()
    }
}
