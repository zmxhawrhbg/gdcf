use crate::{
    api::request::{LevelRequest, LevelRequestType, LevelsRequest, Request, SearchFilters, UserRequest},
    cache::{Cache, CacheEntry, CreatorKey, Lookup, NewgroundsSongKey},
    upgrade::{Upgradable, UpgradeError, UpgradeQuery},
};
use gdcf_model::{
    level::{Level, PartialLevel},
    song::NewgroundsSong,
    user::{Creator, User},
};

impl<Song, User> Upgradable<Level<Song, User>> for PartialLevel<Song, User> {
    type From = PartialLevel<Option<u64>, u64>;
    type LookupKey = LevelRequest;
    type Request = LevelRequest;
    type Upgrade = Level<Option<u64>, u64>;

    fn query_upgrade<C: Cache + Lookup<Self::LookupKey>>(
        &self,
        cache: &C,
        ignored_cached: bool,
    ) -> Result<UpgradeQuery<Self::Request, Self::Upgrade>, UpgradeError<C::Err>> {
        let mut request = LevelRequest::new(self.level_id);

        request.set_force_refresh(ignored_cached);

        query_upgrade!(cache, request, request)
    }

    fn process_query_result<C: Cache + Lookup<Self::LookupKey>>(
        &self,
        cache: &C,
        resolved_query: UpgradeQuery<CacheEntry<Level<Option<u64>, u64>, C::CacheEntryMeta>, Self::Upgrade>,
    ) -> Result<UpgradeQuery<!, Self::Upgrade>, UpgradeError<C::Err>> {
        match resolved_query.one() {
            (None, Some(user)) => Ok(UpgradeQuery::One(None, Some(user))),
            (Some(CacheEntry::Cached(user, _)), _) => Ok(UpgradeQuery::One(None, Some(user))),
            _ => Err(UpgradeError::UpgradeFailed),
        }
    }

    fn upgrade<State>(self, upgrade: UpgradeQuery<State, Self::Upgrade>) -> (Level<Song, User>, UpgradeQuery<State, Self::From>) {
        let upgrade = upgrade.one().1.unwrap();

        let (partial_level, song) = change_partial_level_song(self, ());
        let (partial_level, user) = change_partial_level_user(partial_level, ());

        let (level, song_id) = change_level_song(upgrade, song);
        let (level, creator_id) = change_level_user(level, user);

        let partial_level = change_partial_level_user(partial_level, creator_id).0;
        let partial_level = change_partial_level_song(partial_level, song_id).0;

        (level, UpgradeQuery::One(None, Some(partial_level)))
    }

    fn downgrade<State>(upgraded: Level<Song, User>, downgrade: UpgradeQuery<State, Self::From>) -> (Self, UpgradeQuery<State, Self::Upgrade>) {
        let downgrade = downgrade.one().1.unwrap();

        let (level, song) = change_level_song(upgraded, ());
        let (level, creator) = change_level_user(level, ());

        let (partial_level, song_id) = change_partial_level_song(downgrade, song);
        let (partial_level, creator_id) = change_partial_level_user(partial_level, creator);

        let level = change_level_user(level, creator_id).0;
        let level = change_level_song(level, song_id).0;

        (partial_level, UpgradeQuery::One(None, Some(level)))
    }
}

/*
impl<Song, User> Upgradable<Level<Song, User>> for PartialLevel<Song, User> {
    type From = PartialLevel<Option<u64>, u64>;
    type LookupKey = !;
    type Request = LevelRequest;
    type Upgrade = Level<Option<u64>, u64>;

    fn upgrade_request(&self) -> Option<Self::Request> {
        Some(self.level_id.into())
    }

    fn default_upgrade() -> Option<Self::Upgrade> {
        None
    }

    fn lookup_upgrade<C: Cache + Lookup<Self::LookupKey>>(
        &self,
        _: &C,
        request_result: Level<Option<u64>, u64>,
    ) -> Result<Self::Upgrade, <C as Cache>::Err> {
        Ok(request_result)
    }

    fn upgrade(self, upgrade: Self::Upgrade) -> (Level<Song, User>, Self::From) {
        let (partial_level, song) = change_partial_level_song(self, ());
        let (partial_level, user) = change_partial_level_user(partial_level, ());

        let (level, song_id) = change_level_song(upgrade, song);
        let (level, creator_id) = change_level_user(level, user);

        let partial_level = change_partial_level_user(partial_level, creator_id).0;
        let partial_level = change_partial_level_song(partial_level, song_id).0;

        (level, partial_level)
    }

    fn downgrade(upgraded: Level<Song, User>, downgrade: Self::From) -> (Self, Self::Upgrade) {
        let (level, song) = change_level_song(upgraded, ());
        let (level, creator) = change_level_user(level, ());

        let (partial_level, song_id) = change_partial_level_song(downgrade, song);
        let (partial_level, creator_id) = change_partial_level_user(partial_level, creator);

        let level = change_level_user(level, creator_id).0;
        let level = change_level_song(level, song_id).0;

        (partial_level, level)
    }
}

impl Upgradable<Level<Option<NewgroundsSong>, u64>> for Level<Option<u64>, u64> {
    type From = Option<u64>;
    type LookupKey = NewgroundsSongKey;
    type Request = LevelsRequest;
    type Upgrade = Option<NewgroundsSong>;

    fn upgrade_request(&self) -> Option<Self::Request> {
        match self.base.custom_song {
            Some(song_id) =>
                Some(
                    LevelsRequest::default()
                        .filter(SearchFilters::default().custom_song(song_id))
                        .request_type(LevelRequestType::MostLiked),
                ),
            None => None,
        }
    }

    fn default_upgrade() -> Option<Self::Upgrade> {
        Some(None)
    }

    fn lookup_upgrade<C: Cache + Lookup<Self::LookupKey>>(
        &self,
        cache: &C,
        _: <LevelsRequest as Request>::Result,
    ) -> Result<Self::Upgrade, <C as Cache>::Err> {
        Ok(match self.base.custom_song {
            Some(song_id) => cache.lookup(&NewgroundsSongKey(song_id))?.into(),
            None => None,
        })
    }

    fn upgrade(self, upgrade: Self::Upgrade) -> (Level<Option<NewgroundsSong>, u64>, Self::From) {
        change_level_song(self, upgrade)
    }

    fn downgrade(upgraded: Level<Option<NewgroundsSong>, u64>, downgrade: Self::From) -> (Self, Self::Upgrade) {
        change_level_song(upgraded, downgrade)
    }
}

impl Upgradable<PartialLevel<Option<NewgroundsSong>, u64>> for PartialLevel<Option<u64>, u64> {
    type From = Option<u64>;
    type LookupKey = NewgroundsSongKey;
    type Request = LevelsRequest;
    type Upgrade = Option<NewgroundsSong>;

    fn upgrade_request(&self) -> Option<Self::Request> {
        self.custom_song.map(|song_id| {
            LevelsRequest::default()
                .filter(SearchFilters::default().custom_song(song_id))
                .request_type(LevelRequestType::MostLiked)
        })
    }

    fn default_upgrade() -> Option<Self::Upgrade> {
        Some(None)
    }

    fn lookup_upgrade<C: Cache + Lookup<Self::LookupKey>>(
        &self,
        cache: &C,
        _: <LevelsRequest as Request>::Result,
    ) -> Result<Self::Upgrade, <C as Cache>::Err> {
        Ok(match self.custom_song {
            Some(song_id) => cache.lookup(&NewgroundsSongKey(song_id))?.into(),
            None => None,
        })
    }

    fn upgrade(self, upgrade: Self::Upgrade) -> (PartialLevel<Option<NewgroundsSong>, u64>, Self::From) {
        change_partial_level_song(self, upgrade)
    }

    fn downgrade(upgraded: PartialLevel<Option<NewgroundsSong>, u64>, downgrade: Self::From) -> (Self, Self::Upgrade) {
        change_partial_level_song(upgraded, downgrade)
    }
}

impl<Song> Upgradable<Level<Song, Option<Creator>>> for Level<Song, u64> {
    type From = u64;
    type LookupKey = CreatorKey;
    type Request = LevelsRequest;
    type Upgrade = Option<Creator>;

    fn upgrade_request(&self) -> Option<Self::Request> {
        Some(
            LevelsRequest::default()
                .search(self.base.creator.to_string())
                .request_type(LevelRequestType::User),
        )
    }

    fn default_upgrade() -> Option<Self::Upgrade> {
        Some(None)
    }

    fn lookup_upgrade<C: Cache + Lookup<Self::LookupKey>>(
        &self,
        cache: &C,
        _: Vec<PartialLevel<Option<u64>, u64>>,
    ) -> Result<Self::Upgrade, <C as Cache>::Err> {
        Ok(cache.lookup(&CreatorKey(self.base.creator))?.into())
    }

    fn upgrade(self, upgrade: Self::Upgrade) -> (Level<Song, Option<Creator>>, Self::From) {
        change_level_user(self, upgrade)
    }

    fn downgrade(upgraded: Level<Song, Option<Creator>>, downgrade: Self::From) -> (Self, Self::Upgrade) {
        change_level_user(upgraded, downgrade)
    }
}

impl<Song> Upgradable<PartialLevel<Song, Option<Creator>>> for PartialLevel<Song, u64> {
    type From = u64;
    type LookupKey = CreatorKey;
    type Request = LevelsRequest;
    type Upgrade = Option<Creator>;

    fn upgrade_request(&self) -> Option<Self::Request> {
        Some(
            LevelsRequest::default()
                .search(self.creator.to_string())
                .request_type(LevelRequestType::User),
        )
    }

    fn default_upgrade() -> Option<Self::Upgrade> {
        Some(None)
    }

    fn lookup_upgrade<C: Cache + Lookup<Self::LookupKey>>(
        &self,
        cache: &C,
        _: Vec<PartialLevel<Option<u64>, u64>>,
    ) -> Result<Self::Upgrade, <C as Cache>::Err> {
        Ok(cache.lookup(&CreatorKey(self.creator))?.into())
    }

    fn upgrade(self, upgrade: Self::Upgrade) -> (PartialLevel<Song, Option<Creator>>, Self::From) {
        change_partial_level_user(self, upgrade)
    }

    fn downgrade(upgraded: PartialLevel<Song, Option<Creator>>, downgrade: Self::From) -> (Self, Self::Upgrade) {
        change_partial_level_user(upgraded, downgrade)
    }
}

impl<Song> Upgradable<PartialLevel<Song, Option<User>>> for PartialLevel<Song, Option<Creator>> {
    type From = Option<Creator>;
    type LookupKey = !;
    type Request = UserRequest;
    type Upgrade = Option<User>;

    fn upgrade_request(&self) -> Option<Self::Request> {
        match &self.creator {
            Some(creator) =>
                match creator.account_id {
                    Some(account_id) => Some(account_id.into()),
                    _ => None,
                },
            _ => None,
        }
    }

    fn default_upgrade() -> Option<Self::Upgrade> {
        Some(None)
    }

    fn lookup_upgrade<C: Cache + Lookup<Self::LookupKey>>(&self, _: &C, request_result: User) -> Result<Self::Upgrade, <C as Cache>::Err> {
        Ok(Some(request_result))
    }

    fn upgrade(self, upgrade: Self::Upgrade) -> (PartialLevel<Song, Option<User>>, Self::From) {
        change_partial_level_user(self, upgrade)
    }

    fn downgrade(upgraded: PartialLevel<Song, Option<User>>, downgrade: Self::From) -> (Self, Self::Upgrade) {
        change_partial_level_user(upgraded, downgrade)
    }
}
impl<Song> Upgradable<Level<Song, Option<User>>> for Level<Song, Option<Creator>> {
    type From = Option<Creator>;
    type LookupKey = !;
    type Request = UserRequest;
    type Upgrade = Option<User>;

    fn upgrade_request(&self) -> Option<Self::Request> {
        match &self.base.creator {
            Some(creator) =>
                match creator.account_id {
                    Some(account_id) => Some(account_id.into()),
                    _ => None,
                },
            _ => None,
        }
    }

    fn default_upgrade() -> Option<Self::Upgrade> {
        Some(None)
    }

    fn lookup_upgrade<C: Cache + Lookup<Self::LookupKey>>(&self, _: &C, request_result: User) -> Result<Self::Upgrade, <C as Cache>::Err> {
        Ok(Some(request_result))
    }

    fn upgrade(self, upgrade: Self::Upgrade) -> (Level<Song, Option<User>>, Self::From) {
        change_level_user(self, upgrade)
    }

    fn downgrade(upgraded: Level<Song, Option<User>>, downgrade: Self::From) -> (Self, Self::Upgrade) {
        change_level_user(upgraded, downgrade)
    }
}
*/
fn change_partial_level_song<OldSong, NewSong, User>(
    partial_level: PartialLevel<OldSong, User>,
    new_song: NewSong,
) -> (PartialLevel<NewSong, User>, OldSong) {
    let PartialLevel {
        level_id,
        name,
        description,
        version,
        difficulty,
        downloads,
        main_song,
        gd_version,
        likes,
        length,
        stars,
        featured,
        index_31,
        copy_of,
        coin_amount,
        coins_verified,
        stars_requested,
        index_40,
        is_epic,
        index_43,
        object_amount,
        index_46,
        index_47,
        creator,
        custom_song,
    } = partial_level;

    (
        PartialLevel {
            custom_song: new_song,

            level_id,
            name,
            description,
            version,
            creator,
            difficulty,
            downloads,
            main_song,
            gd_version,
            likes,
            length,
            stars,
            featured,
            index_31,
            copy_of,
            coin_amount,
            coins_verified,
            stars_requested,
            index_40,
            is_epic,
            index_43,
            object_amount,
            index_46,
            index_47,
        },
        custom_song,
    )
}

fn change_partial_level_user<OldUser, NewUser, Song>(
    partial_level: PartialLevel<Song, OldUser>,
    new_user: NewUser,
) -> (PartialLevel<Song, NewUser>, OldUser) {
    let PartialLevel {
        level_id,
        name,
        description,
        version,
        difficulty,
        downloads,
        main_song,
        gd_version,
        likes,
        length,
        stars,
        featured,
        index_31,
        copy_of,
        coin_amount,
        coins_verified,
        stars_requested,
        index_40,
        is_epic,
        index_43,
        object_amount,
        index_46,
        index_47,
        custom_song,
        creator,
    } = partial_level;

    (
        PartialLevel {
            creator: new_user,

            level_id,
            name,
            description,
            version,
            custom_song,
            difficulty,
            downloads,
            main_song,
            gd_version,
            likes,
            length,
            stars,
            featured,
            index_31,
            copy_of,
            coin_amount,
            coins_verified,
            stars_requested,
            index_40,
            is_epic,
            index_43,
            object_amount,
            index_46,
            index_47,
        },
        creator,
    )
}

fn change_level_user<OldUser, NewUser, Song>(level: Level<Song, OldUser>, new_user: NewUser) -> (Level<Song, NewUser>, OldUser) {
    let Level {
        base,
        level_data,
        password,
        time_since_update,
        time_since_upload,
        index_36,
    } = level;

    let (new_base, old_user) = change_partial_level_user(base, new_user);

    (
        Level {
            base: new_base,
            level_data,
            password,
            time_since_update,
            time_since_upload,
            index_36,
        },
        old_user,
    )
}

fn change_level_song<OldSong, NewSong, User>(level: Level<OldSong, User>, new_song: NewSong) -> (Level<NewSong, User>, OldSong) {
    let Level {
        base,
        level_data,
        password,
        time_since_update,
        time_since_upload,
        index_36,
    } = level;

    let (new_base, old_song) = change_partial_level_song(base, new_song);

    (
        Level {
            base: new_base,
            level_data,
            password,
            time_since_update,
            time_since_upload,
            index_36,
        },
        old_song,
    )
}
