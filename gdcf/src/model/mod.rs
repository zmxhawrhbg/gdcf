pub use self::level::{DemonRating, Level, LevelLength, LevelRating, PartialLevel};
pub use self::song::{MainSong, NewgroundsSong};
use std::fmt::{self, Display, Formatter};

mod de;
pub mod level;
pub mod song;
pub mod raw;

/// Enum modelling the version of a Geometry Dash client
#[derive(Debug, Clone, Copy, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub enum GameVersion {
    /// Variant representing an unknown version. This variant is only used for levels that were uploaded
    /// before the game started tracking the version. This variant's string representation is `"10"`
    Unknown,

    /// Variant representing a the version represented by the given minor/major values in the form `major.minor`
    Version { minor: u8, major: u8 },
}

#[derive(Debug)]
pub enum GDObject {
    NewgroundsSong(NewgroundsSong),
    PartialLevel(PartialLevel),
    Level(Level),
}

into_gdo!(Level);
into_gdo!(PartialLevel);
into_gdo!(NewgroundsSong);

impl Display for GDObject {
    fn fmt(&self, f: &mut Formatter) -> Result<(), fmt::Error> {
        match *self {
            GDObject::NewgroundsSong(ref inner) => inner.fmt(f),
            GDObject::PartialLevel(ref inner) => inner.fmt(f),
            GDObject::Level(ref inner) => inner.fmt(f)
        }
    }
}
