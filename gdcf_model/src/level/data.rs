pub mod ids;
pub mod portal;

use crate::level::{
    data::portal::{PortalData, PortalType, Speed},
    Level,
};
use flate2::read::GzDecoder;
use std::{io::Read, time::Duration};

#[derive(Debug, PartialEq, Clone, Default)]
pub struct LevelMetadata {
    pub starting_speed: Speed,
    // ... other fields in the metadata section ...
}

#[derive(Debug, PartialEq, Clone)]
pub struct LevelObject {
    pub id: u16,
    pub x: f32,
    pub y: f32,
    // ... other fields they all have ...
    pub metadata: ObjectData,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ObjectData {
    None,
    Portal(PortalData),
}

#[derive(Debug, PartialEq, Eq)]
pub struct Stats {
    pub duration: Duration,
    pub object_count: u64,
}

impl<S, U> Level<S, U>
where
    U: PartialEq,
    S: PartialEq,
{
    pub fn decompress_data(&self) -> std::io::Result<String> {
        let mut s = String::new();
        let mut d = GzDecoder::new(&self.level_data[..]);

        d.read_to_string(&mut s)?;

        Ok(s)
    }
}

pub struct ParsedLevelData(pub LevelMetadata, pub Vec<LevelObject>);
pub struct ParsedIterator<I>(pub LevelMetadata, pub I)
where
    I: Iterator<Item = LevelObject>;

impl<I> IntoIterator for ParsedIterator<I>
where
    I: Iterator<Item = LevelObject>,
{
    type IntoIter = I;
    type Item = LevelObject;

    fn into_iter(self) -> I {
        self.1
    }
}

impl<I> ParsedIterator<I>
where
    I: Iterator<Item = LevelObject>,
{
    /// Calculates as many stats about the level as possible in a single iteration pass.
    pub fn stats(ParsedIterator(metadata, iter): Self) -> Stats {
        let mut object_count = 0;
        let mut portals = Vec::new();
        let mut furthest_x = 0.0;

        for object in iter {
            object_count += 1;

            if let ObjectData::Portal(PortalData {
                checked: true,
                portal_type: PortalType::Speed(speed),
            }) = object.metadata
            {
                portals.push((object.x, speed))
            }

            furthest_x = f32::max(furthest_x, object.x);
        }

        portals.sort_by(|(x1, _), (x2, _)| x1.partial_cmp(x2).unwrap());

        let duration = Duration::from_secs(portal::get_seconds_from_x_pos(furthest_x, metadata.starting_speed, &portals) as u64);

        Stats { object_count, duration }
    }

    pub fn collect(self) -> ParsedLevelData {
        ParsedLevelData(self.0, self.1.collect())
    }
}