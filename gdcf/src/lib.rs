#![feature(box_syntax)]
#![feature(never_type)]
#![feature(try_from)]
#![deny(
    bare_trait_objects,
    missing_debug_implementations,
    unused_extern_crates,
    patterns_in_fns_without_body,
    stable_features,
    unknown_lints,
    unused_features,
    unused_imports,
    unused_parens
)]

//! The `gdcf` crate is the core of the Geometry Dash Caching Framework.
//! It provides all the core traits required to implement an API Client and
//! a cache which are used by implementations of the [`Gdcf`] trait.
//!
//! [`Gdcf`]: trait.Gdcf.html
//!
//! # Geometry Dash Caching Framework
//!
//! The idea behind the Geometry Dash Caching Framework is to provide fast and
//! reliable access to the resources provided by the Geometry Dash servers. It
//! achieves this goal by caching all responses from the servers and only
//! returning those cached responses when a
//! request is attempted, while refreshing the cache asynchronously, in the
//! background. This ensures instant access to information such as level
//! description that can be used easily
//! even in environments where the slow response times and unreliable
//! availability of RobTop's server would be
//! unacceptable otherwise
//!
//! It further ensures the integrity of its cached data, which means it
//! automatically generates more requests if it notices that, i.e., a level you
//! just retrieved doesn't have its newgrounds song
//! cached.
//!
extern crate base64;
extern crate chrono;
extern crate futures;
#[macro_use]
extern crate gdcf_derive;
extern crate joinery;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate log;
extern crate percent_encoding;
#[cfg(feature = "deser")]
extern crate serde;
#[cfg(feature = "deser")]
#[macro_use]
extern crate serde_derive;

use api::{
    request::{level::SearchFilters, LevelRequest, LevelsRequest, StreamableRequest},
    response::ProcessedResponse,
    ApiClient,
};
use cache::Cache;
use error::{CacheError, GdcfError};
use futures::{
    future::{join_all, result, Either},
    task, Async, Future, Stream,
};
use model::{GDObject, Level, PartialLevel};
use std::{
    error::Error,
    mem,
    sync::{Arc, Mutex},
};
use error::ApiError;

#[macro_use]
mod macros;

pub mod api;
pub mod cache;
pub mod convert;
pub mod error;
pub mod model;

#[derive(Debug)]
pub struct Gdcf<A: ApiClient + 'static, C: Cache + 'static> {
    client: Arc<Mutex<A>>,
    cache: Arc<Mutex<C>>,
}

impl<A: ApiClient + 'static, C: Cache + 'static> Clone for Gdcf<A, C> {
    fn clone(&self) -> Self {
        Gdcf {
            client: self.client.clone(),
            cache: self.cache.clone(),
        }
    }
}

impl<A: ApiClient + 'static, C: Cache + 'static> Gdcf<A, C> {
    gdcf_one!(level, LevelRequest, Level, lookup_level, level_future);

    gdcf_many!(
        levels,
        LevelsRequest,
        PartialLevel,
        lookup_partial_levels,
        store_partial_levels,
        levels_future
    );

    pub fn levels_stream(&self, req: LevelsRequest) -> GdcfStream<LevelsRequest, Vec<PartialLevel>, A, C> {
        GdcfStream::new(self.clone(), req, Self::levels)
    }

    pub fn new(client: A, cache: C) -> Gdcf<A, C> {
        Gdcf {
            client: Arc::new(Mutex::new(client)),
            cache: Arc::new(Mutex::new(cache)),
        }
    }

    fn integrity(
        &self, response: ProcessedResponse,
    ) -> impl Future<Item = ProcessedResponse, Error = GdcfError<A::Err, C::Err>> + Send + 'static {
        let mut reqs = Vec::new();

        for obj in &response {
            match obj {
                GDObject::Level(level) =>
                    if let Some(song_id) = level.base.custom_song_id {
                        match lock!(self.cache).lookup_song(song_id) {
                            Err(CacheError::CacheMiss) => {
                                warn!("Integrity request required to gather newgrounds song with ID {}", song_id);

                                reqs.push(
                                    self.levels(
                                        LevelsRequest::default()
                                            .with_id(level.base.level_id)
                                            .filter(SearchFilters::default().custom_song(song_id)),
                                    ).map(|_| ()),
                                )
                            },

                            Err(err) => return Either::B(result(Err(GdcfError::Cache(err)))),

                            _ => continue,
                        }
                    },
                _ => (),
            }
        }

        if reqs.is_empty() {
            Either::B(result(Ok(response)))
        } else {
            Either::A(join_all(reqs).map(move |_| response))
        }
    }
}

#[allow(missing_debug_implementations)]
pub struct GdcfFuture<T, AE: Error + Send + 'static, CE: Error + Send + 'static> {
    // invariant: at least one of the fields is not `None`
    cached: Option<T>,
    refresher: Option<Box<dyn Future<Item = T, Error = GdcfError<AE, CE>> + Send + 'static>>,
}

impl<T, CE: Error + Send + 'static, AE: Error + Send + 'static> GdcfFuture<T, AE, CE> {
    fn up_to_date(object: T) -> GdcfFuture<T, AE, CE> {
        GdcfFuture {
            cached: Some(object),
            refresher: None,
        }
    }

    fn outdated<F>(object: T, f: F) -> GdcfFuture<T, AE, CE>
    where
        F: Future<Item = T, Error = GdcfError<AE, CE>> + Send + 'static,
    {
        GdcfFuture {
            cached: Some(object),
            refresher: Some(Box::new(f)),
        }
    }

    fn absent<F>(f: F) -> GdcfFuture<T, AE, CE>
    where
        F: Future<Item = T, Error = GdcfError<AE, CE>> + Send + 'static,
    {
        GdcfFuture {
            cached: None,
            refresher: Some(Box::new(f)),
        }
    }

    pub fn cached(&self) -> &Option<T> {
        &self.cached
    }

    pub fn take(&mut self) -> Option<T> {
        mem::replace(&mut self.cached, None)
    }
}

impl<T, AE: Error + Send + 'static, CE: Error + Send + 'static> Future for GdcfFuture<T, AE, CE> {
    type Error = GdcfError<AE, CE>;
    type Item = T;

    fn poll(&mut self) -> Result<Async<T>, GdcfError<AE, CE>> {
        match self.refresher {
            Some(ref mut fut) => fut.poll(),
            None => Ok(Async::Ready(self.take().unwrap())),
        }
    }
}

#[allow(missing_debug_implementations)]
pub struct GdcfStream<S, N, A, C>
where
    S: StreamableRequest,
    A: ApiClient + 'static,
    C: Cache + 'static,
{
    gdcf: Gdcf<A, C>,
    request: S,
    current: GdcfFuture<N, A::Err, C::Err>,
    request_maker: Box<dyn Fn(&Gdcf<A, C>, S) -> GdcfFuture<N, A::Err, C::Err> + Send + 'static>,
}

impl<S, N, A, C> GdcfStream<S, N, A, C>
where
    S: StreamableRequest,
    A: ApiClient + 'static,
    C: Cache + 'static,
{
    pub fn new<F: 'static>(gdcf: Gdcf<A, C>, request: S, func: F) -> GdcfStream<S, N, A, C>
    where
        F: Fn(&Gdcf<A, C>, S) -> GdcfFuture<N, A::Err, C::Err> + Send,
    {
        let next = request.next();
        let current = func(&gdcf, request);

        GdcfStream {
            gdcf,
            request: next,
            current,
            request_maker: Box::new(func),
        }
    }
}

impl<S, N, A, C> Stream for GdcfStream<S, N, A, C>
where
    S: StreamableRequest,
    A: ApiClient + 'static,
    C: Cache + 'static,
{
    type Error = GdcfError<A::Err, C::Err>;
    type Item = N;

    fn poll(&mut self) -> Result<Async<Option<N>>, GdcfError<A::Err, C::Err>> {
        match self.current.poll() {
            Ok(Async::NotReady) => Ok(Async::NotReady),

            Ok(Async::Ready(result)) => {
                task::current().notify();

                let next = self.request.next();
                let cur = mem::replace(&mut self.request, next);

                self.current = (self.request_maker)(&self.gdcf, cur);

                Ok(Async::Ready(Some(result)))
            },

            Err(GdcfError::NoContent) | Err(GdcfError::Api(ApiError::NoData)) => Ok(Async::Ready(None)), // We're done here

            Err(err) => Err(err),
        }
    }
}
