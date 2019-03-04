use crate::{
    util::{b64_decode_bytes, b64_decode_string, default_to_none, int_to_bool, xor_decrypt},
    Parse,
};
use base64::DecodeError;
use gdcf::{
    error::ValueError,
    model::{
        song::{MainSong, MAIN_SONGS, UNKNOWN},
        Level, LevelRating, PartialLevel, Password,
    },
};

pub fn process_difficulty(rating: i32, is_auto: bool, is_demon: bool) -> LevelRating {
    if is_demon {
        LevelRating::Demon(rating.into())
    } else if is_auto {
        LevelRating::Auto
    } else {
        rating.into()
    }
}

pub fn process_song(main_song: usize, custom_song: &Option<u64>) -> Option<&'static MainSong> {
    if custom_song.is_none() {
        Some(MAIN_SONGS.get(main_song).unwrap_or(&UNKNOWN))
    } else {
        None
    }
}

pub fn parse_description(value: &str) -> Option<String> {
    // I have decided that level descriptions are so broken that we simply ignore it if they fail to
    // parase
    b64_decode_string(value).ok()
}

/// Attempts to parse the given `str` into a [`Password`]
///
/// # Errors
/// If the given string isn't `"0"` and also isn't valid URL-safe base64, a
/// [`DecodeError`] is returned
pub fn level_password(encrypted: &str) -> Result<Password, DecodeError> {
    match encrypted {
        "0" => Ok(Password::NoCopy),
        pass => {
            let decoded = b64_decode_string(pass)?;
            let mut decrypted = xor_decrypt(&decoded, "26364");

            if decrypted.len() == 1 {
                Ok(Password::FreeCopy)
            } else {
                decrypted.remove(0);
                Ok(Password::PasswordCopy(decrypted))
            }
        },
    }
}

parser! {
    PartialLevel<u64, u64> => {
        level_id(index = 1),
        name(index = 2),
        description(index = 3, parse_infallible = parse_description, default),
        version(index = 5),
        creator(index = 6),
        difficulty(custom = process_difficulty, depends_on = [rating, is_auto, is_demon]),
        downloads(index = 10),
        main_song(custom = process_song, depends_on = [main_song_id, &custom_song]),
        gd_version(index = 13),
        likes(index = 14),
        length(index = 15),
        stars(index = 18),
        featured(index = 19),
        copy_of(index = 30, with = default_to_none),
        custom_song(index = 35, with = default_to_none),
        coin_amount(index = 37),
        coins_verified(index = 38, with = int_to_bool),
        stars_requested(index = 39, with = default_to_none),
        is_epic(index = 42, with = int_to_bool),
        index_43(index = 43),
        object_amount(index = 45),
        index_46(index = 46),
        index_47(index = 47),
    },
    main_song_id(index = 12, default),
    rating(index = 9),
    is_demon(index = 17, with = int_to_bool, default),
    is_auto(index = 25, with = int_to_bool, default),
}

parser! {
    Level<u64, u64> => {
        base(delegate),
        level_data(index = 4, parse = b64_decode_bytes),
        password(index = 27, parse = level_password),
        time_since_upload(index = 28),
        time_since_update(index = 29),
        index_36(index = 36, default),
    }
}
