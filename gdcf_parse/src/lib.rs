//! Crate containing parsers for various Geometry Dash related data
//!
//! This crate is based on work by mgostIH and cos8o
#![feature(pattern)]

use crate::util::SelfZipExt;
use gdcf::error::ValueError;
use std::str::pattern::Pattern;

pub mod util;
#[macro_use]
pub mod macros;
pub mod level;
pub mod level_data;
pub mod song;
pub mod user;

const INDICES: [&str; 50] = [
    "1", "2", "3", "4", "5", "6", "7", "8", "9", "10", "11", "12", "13", "14", "15", "16", "17", "18", "19", "20", "21", "22", "23", "24",
    "25", "26", "27", "28", "29", "30", "31", "32", "33", "34", "35", "36", "37", "38", "39", "40", "41", "42", "43", "44", "45", "46",
    "47", "48", "49", "50",
];

pub trait Parse: Sized {
    fn parse<'a, I, F>(iter: I, f: F) -> Result<Self, ValueError<'a>>
    where
        I: Iterator<Item = (&'a str, &'a str)> + Clone,
        F: FnMut(&'a str, &'a str) -> Result<(), ValueError<'a>>;

    fn parse_iter<'a>(iter: impl Iterator<Item = &'a str> + Clone) -> Result<Self, ValueError<'a>> {
        Self::parse(iter.self_zip(), |_, _| Ok(())) // TODO: warnings about unused indices
    }

    fn parse_unindexed_iter<'a>(iter: impl Iterator<Item = &'a str> + Clone) -> Result<Self, ValueError<'a>> {
        // well this is a stupid solution
        Self::parse(INDICES.iter().cloned().zip(iter), |_, _| Ok(()))
    }

    fn parse_str<'a, P>(input: &'a str, delimiter: P) -> Result<Self, ValueError<'a>>
    where
        P: Pattern<'a> + Clone,
        P::Searcher: Clone,
    {
        Self::parse_iter(input.split(delimiter))
    }

    fn parse_unindexed_str<'a, P>(input: &'a str, delimiter: P) -> Result<Self, ValueError<'a>>
    where
        P: Pattern<'a> + Clone,
        P::Searcher: Clone,
    {
        Self::parse_unindexed_iter(input.split(delimiter))
    }
}