use convert;
use error::ValueError;
use model::level::Featured;
use model::LevelRating;
use model::raw::RawObject;
use model::song::{MAIN_SONGS, MainSong, UNKNOWN};
use percent_encoding::percent_decode;
use std::num::ParseIntError;
use std::str::{FromStr, Utf8Error};
use base64::DecodeError;

pub(super) fn level_rating(raw_obj: &RawObject) -> Result<LevelRating, ValueError> {
    let is_demon = raw_obj.get_with_or(17, int_to_bool, false)?;
    let is_auto = raw_obj.get_with_or(25, int_to_bool, false)?;
    let rating: i32 = raw_obj.get(9)?;

    if is_demon {
        Ok(LevelRating::Demon(rating.into()))
    } else if is_auto {
        Ok(LevelRating::Auto)
    } else {
        Ok(rating.into())
    }
}

pub(super) fn main_song(raw_obj: &RawObject) -> Result<Option<&'static MainSong>, ValueError> {
    if raw_obj.get::<u64>(35)? == 0 {
        Ok(Some(
            MAIN_SONGS
                .get(raw_obj.get::<usize>(12)?)
                .unwrap_or(&UNKNOWN),
        ))
    } else {
        Ok(None)
    }
}

pub(super) fn description(value: &str) -> Result<Option<String>, DecodeError> {
    convert::to::b64_decoded_string(value)
        .map(Option::Some)
}

pub(super) fn default_to_none<T>(value: &str) -> Result<Option<T>, <T as FromStr>::Err>
    where
        T: FromStr + Default + PartialEq,
{
    let value: T = value.parse()?;

    if value == Default::default() {
        Ok(None)
    } else {
        Ok(Some(value))
    }
}

pub(super) fn int_to_bool(value: &str) -> Result<bool, ParseIntError> {
    Ok(convert::to::bool(value.parse()?))
}

pub(super) fn into_option(value: &str) -> Result<Option<String>, !> {
    Ok(Some(value.to_string()))
}
