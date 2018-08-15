//! Module containing request definitions for retrieving levels
//!
//! Note that all `Hash` impls are to be forward compatible with new fields in the request.
//! This means, that if an update to the GD API arrives which adds more fields to a request,
//! those fields are hashed _only_ if they are different from their default values.
//! This way, the hashes of requests made before the update will stay the same

use api::ApiClient;
use api::client::ApiFuture;
use api::request::{BaseRequest, Request};
use model::{DemonRating, LevelLength, LevelRating};
#[cfg(feature = "deser")]
use serde::{Deserialize, Serialize, Serializer};
use std::fmt::{Display, Error, Formatter};
use std::hash::Hash;
use std::hash::Hasher;

/// Struct modelled after a request to `downloadGJLevel22.php`.
///
/// In the Geometry Dash API, this endpoint is used to download a level from the servers
/// and retrieve some additional information that isn't provided with the response to a
/// [LevelsRequest](struct.LevelsRequest.html)
#[derive(Debug, Default)]
pub struct LevelRequest {
    /// The base request data
    pub base: BaseRequest,

    /// The ID of the level to download
    ///
    /// ## GD Internals:
    /// This field is called `levelID` in the boomlings API
    pub level_id: u64,

    /// Some weird field the Geometry Dash Client sends along
    ///
    /// ## GD Internals:
    /// This value needs to be converted to an integer for the boomlings API
    pub inc: bool,

    /// Some weird field the Geometry Dash Client sends along
    ///
    /// ## GD Internals:
    /// This field is called `extras` in the boomlings API and needs to be converted to an integer
    pub extra: bool,
}

/// Manual `Hash` impl that doesn't hash `base`.
impl Hash for LevelRequest {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.level_id.hash(state);
        self.inc.hash(state);
        self.extra.hash(state);
    }
}

/// Struct modelled after a request to `getGJLevels21.php`
///
/// In the Geometry Dash API, this endpoint is used to retrieve a list of levels matching
/// the specified criteria, along with their [NewgroundsSong](../../../model/song/struct.NewgroundsSong.html)s
/// and some basic information on their creators.
#[derive(Debug, Default, Clone)]
pub struct LevelsRequest {
    /// The base request data
    pub base: BaseRequest,

    /// The type of level list to retrieve
    ///
    /// ## GD Internals:
    /// This field is called `type` in the boomlings API and needs to be converted to an integer
    pub request_type: LevelRequestType,

    /// A search string to filter the levels by
    ///
    /// This value is ignored unless [request_type](struct.LevelsRequest.html#structfield.request_type)
    /// is set to [Search](enum.LevelRequestType.html#variant.Search)
    ///
    /// ## GD Internals:
    /// This field is called `str` in the boomlings API
    pub search_string: String,

    /// A list of level lengths to filter by
    ///
    /// This value is ignored unless [request_type](struct.LevelsRequest.html#structfield.request_type)
    /// is set to [Search](enum.LevelRequestType.html#variant.Search)
    ///
    /// ## GD Internals:
    /// This field is called `len` in the boomlings API and needs to be converted to a
    /// comma separated list of integers, or a single dash (`-`) if filtering by level length isn't
    /// wanted.
    pub lengths: Vec<LevelLength>,

    /// A list of level ratings to filter by.
    ///
    /// To filter by any demon, add [LevelRating::Demon(_)](../../../model/level/enum.LevelRating.html#variant.Demon)
    /// with any arbitrary [DemonRating](../../../model/level/enum.DemonRating.html) value.
    ///
    /// `ratings` and [demon_rating](struct.LevelsRequest.html#structfield.demon_rating) are
    /// mutually exlusive.
    ///
    /// This value is ignored unless [request_type](struct.LevelsRequest.html#structfield.request_type)
    /// is set to [Search](enum.LevelRequestType.html#variant.Search)
    ///
    /// ## GD Internals:
    /// This field is called `diff` in the boomlings API and needs to be converted to a
    /// comma separated list of integers, or a single dash (`-`) if filtering by level rating isn't
    /// wanted.
    pub ratings: Vec<LevelRating>,

    /// Optionally, a single demon rating to filter by. To filter by any demon rating, use
    /// [ratings](struct.LevelsRequest.html#structfield.ratings)
    ///
    /// `demon_rating` and `ratings` are mutually exlusive.
    ///
    /// This value is ignored unless [request_type](struct.LevelsRequest.html#structfield.request_type)
    /// is set to [Search](enum.LevelRequestType.html#variant.Search)
    ///
    /// ## GD Internals:
    /// This field is called `demonFilter` in the boomlings API and needs to be converted to
    /// an integer. If filtering by demon rating isn't wanted, the value has to be omitted from the
    /// request.
    pub demon_rating: Option<DemonRating>,

    /// The page of results to retrieve
    pub page: u32,

    /// Some weird value the Geometry Dash client sends along
    pub total: i32,

    /// Search filters to apply.
    ///
    /// These values is ignored unless [request_type](struct.LevelsRequest.html#structfield.request_type)
    /// is set to [Search](enum.LevelRequestType.html#variant.Search)
    pub search_filters: SearchFilters,
}

/// Manual Hash impl which doesn't hash the base
impl Hash for LevelsRequest {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.search_filters.hash(state);
        self.total.hash(state);
        self.demon_rating.hash(state);
        self.ratings.hash(state);
        self.lengths.hash(state);
        self.search_string.hash(state);
        self.request_type.hash(state);
        self.page.hash(state);
    }
}

/// Struct containing the various search filters provided by the Geometry Dash client.
#[derive(Debug, Default, Copy, Clone, Hash)]
pub struct SearchFilters {
    /// Only retrieve uncompleted levels
    ///
    /// ## GD Internals:
    /// This value needs to be converted to an integer for the boomlings API
    pub uncompleted: bool,

    /// Only retrieve completed levels
    ///
    /// ## GD Internals:
    /// This field is called `onlyCompleted` in the boomlings API and needs to be converted to
    /// an integer
    pub completed: bool,

    /// Only retrieve featured levels
    ///
    /// ## GD Internals:
    /// This value needs to be converted to an integer for the boomlings API
    pub featured: bool,

    /// Only retrieve original (uncopied)  levels
    ///
    /// ## GD Internals:
    /// This value needs to be converted to an integer for the boomlings API
    pub original: bool,

    /// Only retrieve two-player levels
    ///
    /// ## GD Internals:
    /// This field is called `twoPlayer` in the boomlings API and needs to be converted to
    /// an integer
    pub two_player: bool,

    /// Only retrieve levels with coins
    ///
    /// ## GD Internals:
    /// This value needs to be converted to an integer for the boomlings API
    pub coins: bool,

    /// Only retrieve epic levels
    ///
    /// ## GD Internals:
    /// This value needs to be converted to an integer for the boomlings API
    pub epic: bool,

    /// Only retrieve star rated levels
    ///
    /// ## GD Internals:
    /// This field is called `star` in the boomlings API and needs to be converted to
    /// an integer
    pub rated: bool,

    /// Optionally only retrieve levels that match the given `SongFilter`
    ///
    /// ## GD Internals:
    /// This field composes both the `customSong` and `song` fields of the boomlings API.
    /// To filter by main song, set the `song` field to the id of the main song, and omit the `customSong`
    /// field from the request. To filter by a newgrounds song, set `customSong` to `1` and `song`
    /// to the newgrounds ID of the custom song.
    pub song: Option<SongFilter>,
}

/// Enum containing the various types of [LevelsRequest](struct.LevelsRequest.html) possible
#[derive(Debug, Copy, Clone, PartialOrd, PartialEq, Hash)]
pub enum LevelRequestType {
    /// A search request.
    ///
    /// Setting this variant will enabled all the available search filters
    ///
    /// ## GD Internals:
    /// This variant is represented by the value `0` in requests
    Search,

    /// Request to retrieve the list of most downloaded levels
    ///
    /// ## GD Internals:
    /// This variant is represented by the value `1` in requests
    MostDownloaded,

    /// Request to retrieve the list of most liked levels
    ///
    /// ## GD Internals:
    /// This variant is represented by the value `2` in requests
    MostLiked,

    /// Request to retrieve the list of trending levels
    ///
    /// ## GD Internals:
    /// This variant is represented by the value `3` in requests
    Trending,

    /// Request to retrieve the list of most recent levels
    ///
    /// ## GD Internals:
    /// This variant is represented by the value `4` in requests
    Recent,

    /// Unknown how this works
    ///
    /// ## GD Internals:
    /// This variant is represented by the value `5` in requests
    User,

    /// Request to retrieve the list of featured levels, ordered by their [featured weight](../../../model/level/enum.Featured.html)
    ///
    /// ## GD Internals:
    /// This variant is represented by the value `6` in requests
    Featured,

    /// Request to retrieve a list of levels filtered by some magic criteria
    ///
    /// ## GD Internals:
    /// This variant is represented by the value `7` in requests
    Magic,

    /// Unknown what this is
    ///
    /// ## GD Internals:
    /// This variant is represented by the value `8` in requests
    Unknown8,

    /// Request to retrieve the list of levels most recently awarded a rating
    ///
    /// ## GD Internals:
    /// This variant is represented by the value `9` in requests
    Awarded,

    /// Unknown how this works (MapPack according to GDPS source)
    ///
    /// ## GD Internals:
    /// This variant is represented by the value `10` in requests
    Followed,

    /// Unknown how this works
    ///
    /// ## GD Internals:
    /// This variant is represented by the value `11` in requests
    Friend,

    /// Unknown what this is (Followed according to GDPS source)
    ///
    /// ## GD Internals:
    /// This variant is represented by the value `12` in requests
    Unknown12,

    /// Unknown what this is (Friends according to GDPS source)
    ///
    /// ## GD Internals:
    /// This variant is represented by the value `13` in requests
    Unknown13,

    /// Unknown what this is
    ///
    /// ## GD Internals:
    /// This variant is represented by the value `14` in requests
    Unknown14,

    /// Unknown what this is
    ///
    /// ## GD Internals:
    /// This variant is represented by the value `15` in requests
    Unknown15,

    /// Request to retrieve the levels in the hall of fame
    ///
    /// ## GD Internals:
    /// This variant is represented by the value `16` in requests.
    HallOfFame,
}

#[derive(Debug, Copy, Clone, Hash)]
pub enum SongFilter {
    Main(u8),
    Custom(u64),
}

impl SearchFilters {
    pub fn new() -> SearchFilters {
        SearchFilters::default()
    }

    pub fn rated(mut self) -> SearchFilters {
        self.rated = true;
        self
    }

    pub fn uncompleted(mut self) -> SearchFilters {
        self.uncompleted = true;
        self
    }

    pub fn completed(mut self) -> SearchFilters {
        self.completed = true;
        self
    }

    pub fn featured(mut self) -> SearchFilters {
        self.featured = true;
        self
    }

    pub fn original(mut self) -> SearchFilters {
        self.original = true;
        self
    }

    pub fn two_player(mut self) -> SearchFilters {
        self.two_player = true;
        self
    }

    pub fn coins(mut self) -> SearchFilters {
        self.coins = true;
        self
    }

    pub fn epic(mut self) -> SearchFilters {
        self.epic = true;
        self
    }

    pub fn main_song(mut self, id: u8) -> SearchFilters {
        self.song = Some(SongFilter::Main(id));
        self
    }

    pub fn custom_song(mut self, id: u64) -> SearchFilters {
        self.song = Some(SongFilter::Custom(id));
        self
    }
}

impl LevelRequest {
    /// Constructs a new `LevelRequest` to retrieve the level with the given id
    ///
    /// Uses a default [BaseRequest](../struct.BaseRequest.html), and sets the `inc` field to
    /// `true` and `extra` to `false`, as are the default values set the by the Geometry Dash Client
    pub fn new(level_id: u64) -> LevelRequest {
        LevelRequest {
            base: BaseRequest::default(),
            level_id,
            inc: true,
            extra: false,
        }
    }

    setter! {
        /// Sets the `BaseRequest` to be used
        ///
        /// Allows builder-style creation of requests
        base[with_base]: BaseRequest
    }

    setter! {
        /// Sets the value of the `inc` field
        ///
        /// Allows builder-style creation of requests
        inc: bool
    }

    setter! {
        /// Sets the value of the `extra` field
        ///
        /// Allows builder-style creation of requests
        extra: bool
    }
}

impl LevelsRequest {
    setter!(with_base, base, BaseRequest);

    setter!(filter, search_filters, SearchFilters);
    setter!(page, u32);
    setter!(total, i32);
    setter!(request_type, LevelRequestType);

    pub fn search(mut self, search_string: String) -> Self {
        self.search_string = search_string;
        self.request_type = LevelRequestType::Search;
        self
    }

    pub fn with_id(self, id: u64) -> Self {
        self.search(id.to_string())
    }

    pub fn with_length(mut self, length: LevelLength) -> Self {
        self.lengths.push(length);
        self
    }

    pub fn with_rating(mut self, rating: LevelRating) -> Self {
        self.ratings.push(rating);
        self
    }

    pub fn demon(mut self, demon_rating: DemonRating) -> Self {
        self.demon_rating = Some(demon_rating);
        self
    }
}

impl Default for LevelRequestType {
    fn default() -> LevelRequestType {
        LevelRequestType::Featured
    }
}

impl From<LevelRequestType> for i32 {
    fn from(req_type: LevelRequestType) -> Self {
        match req_type {
            LevelRequestType::Search => 0,
            LevelRequestType::MostDownloaded => 1,
            LevelRequestType::MostLiked => 2,
            LevelRequestType::Trending => 3,
            LevelRequestType::Recent => 4,
            LevelRequestType::User => 5,
            LevelRequestType::Featured => 6,
            LevelRequestType::Magic => 7,
            LevelRequestType::Unknown8 => 8,
            LevelRequestType::Awarded => 9,
            LevelRequestType::Followed => 10,
            LevelRequestType::Friend => 11,
            LevelRequestType::Unknown12 => 12,
            LevelRequestType::Unknown13 => 13,
            LevelRequestType::Unknown14 => 14,
            LevelRequestType::Unknown15 => 15,
            LevelRequestType::HallOfFame => 16,
        }
    }
}

impl From<u64> for LevelRequest {
    fn from(lid: u64) -> Self {
        LevelRequest::new(lid)
    }
}

impl Request for LevelRequest {
    fn make<C: ApiClient>(&self, client: &C) -> ApiFuture<C::Err> {
        client.level(&self)
    }
}

impl Request for LevelsRequest {
    fn make<C: ApiClient>(&self, client: &C) -> ApiFuture<C::Err> {
        client.levels(&self)
    }
}

impl Display for LevelRequest {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        write!(f, "LevelRequest({})", self.level_id)
    }
}

impl Display for LevelsRequest {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        match self.request_type {
            LevelRequestType::Search => write!(f, "LevelsRequest(Search={}, page={})", self.search_string, self.page),
            _ => write!(f, "LevelsRequest({:?}, page={})", self.request_type, self.page),
        }
    }
}