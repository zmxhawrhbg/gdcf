use futures::{Async, Future};
use log::trace;

use gdcf_model::{song::NewgroundsSong, user::Creator};

use crate::{
    api::{
        client::MakeRequest,
        request::{PaginatableRequest, Request},
        ApiClient,
    },
    cache::{Cache, CacheEntry, CanCache, CreatorKey, Lookup, NewgroundsSongKey, Store},
    error::Error,
    future::{refresh::RefreshCacheFuture, upgrade::UpgradeFuture, PeekableFuture, StreamableFuture},
    upgrade::Upgradable,
    Gdcf,
};

pub struct ProcessRequestFuture<Req, A, C>
where
    A: ApiClient + MakeRequest<Req>,
    C: Cache + Store<CreatorKey> + Store<NewgroundsSongKey> + CanCache<Req>,
    Req: Request,
{
    gdcf: Gdcf<A, C>,
    forces_refresh: bool,
    state: ProcessRequestFutureState<Req, A, C>,
}

impl<Req, A, C> ProcessRequestFuture<Req, A, C>
where
    A: ApiClient + MakeRequest<Req>,
    C: Cache + Store<CreatorKey> + Store<NewgroundsSongKey> + CanCache<Req>,
    Req: Request,
{
    pub fn new(gdcf: Gdcf<A, C>, request: Req) -> Result<Self, C::Err> {
        Ok(ProcessRequestFuture {
            forces_refresh: request.forces_refresh(),
            state: gdcf.process(request)?,
            gdcf,
        })
    }
}

impl<Req, A, C> StreamableFuture<A, C> for ProcessRequestFuture<Req, A, C>
where
    A: ApiClient + MakeRequest<Req>,
    C: Cache + Store<CreatorKey> + Store<NewgroundsSongKey> + CanCache<Req>,
    Req: PaginatableRequest,
{
    fn next(self, gdcf: &Gdcf<A, C>) -> Result<Self, Self::Error> {
        let mut request = match self.state {
            ProcessRequestFutureState::UpToDate(_, request) => request,
            ProcessRequestFutureState::Outdated(_, future) | ProcessRequestFutureState::Uncached(future) => future.request,
        };
        request.next();
        Ok(ProcessRequestFuture {
            gdcf: self.gdcf,
            forces_refresh: self.forces_refresh,
            state: gdcf.process(request).map_err(Error::Cache)?,
        })
    }
}

impl<Req, A, C> PeekableFuture for ProcessRequestFuture<Req, A, C>
where
    A: ApiClient + MakeRequest<Req>,
    C: Cache + Store<CreatorKey> + Store<NewgroundsSongKey> + CanCache<Req>,
    Req: Request,
{
    fn peek<F: FnOnce(Self::Item) -> Result<Self::Item, Self::Error>>(self, f: F) -> Result<Self, Self::Error> {
        let ProcessRequestFuture {
            gdcf,
            forces_refresh,
            state,
        } = self;

        trace!("State before executing peek_cached closure: {:?}", state);

        let state = match state {
            ProcessRequestFutureState::Outdated(cache_entry, future) => ProcessRequestFutureState::Outdated(f(cache_entry)?, future),
            ProcessRequestFutureState::UpToDate(Some(cache_entry), request) =>
                ProcessRequestFutureState::UpToDate(Some(f(cache_entry)?), request),
            _ => state,
        };

        trace!("State after executing peek_cached closure: {:?}", state);

        Ok(ProcessRequestFuture {
            state,
            gdcf,
            forces_refresh,
        })
    }

    /*fn can_peek(&self) -> bool {
        match self.state {
            ProcessRequestFutureState::Outdated(..) | ProcessRequestFutureState::UpToDate(_) => true,
            _ => false,
        }
    }*/
}

impl<Req, A, C> Future for ProcessRequestFuture<Req, A, C>
where
    A: ApiClient + MakeRequest<Req>,
    C: Cache + Store<CreatorKey> + Store<NewgroundsSongKey> + CanCache<Req>,
    Req: Request,
{
    type Error = Error<A::Err, C::Err>;
    type Item = CacheEntry<Req::Result, C::CacheEntryMeta>;

    fn poll(&mut self) -> Result<Async<Self::Item>, Self::Error> {
        match &mut self.state {
            ProcessRequestFutureState::UpToDate(None, _) => panic!("Future already polled to completion"),
            ProcessRequestFutureState::Uncached(future) => future.poll(),
            ProcessRequestFutureState::Outdated(_, future) => future.poll(),
            ProcessRequestFutureState::UpToDate(cache_entry, _) => Ok(Async::Ready(cache_entry.take().unwrap())),
        }
    }
}

impl<Req, A, C> std::fmt::Debug for ProcessRequestFuture<Req, A, C>
where
    A: ApiClient + MakeRequest<Req>,
    C: Cache + Store<CreatorKey> + Store<NewgroundsSongKey> + CanCache<Req>,
    Req: Request + std::fmt::Debug,
{
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        fmt.debug_struct("ProcessRequestFuture")
            .field("forces_refresh", &self.forces_refresh)
            .field("state", &self.state)
            .finish()
    }
}

pub(crate) enum ProcessRequestFutureState<Req, A, C>
where
    A: ApiClient + MakeRequest<Req>,
    C: Cache + Store<CreatorKey> + Store<NewgroundsSongKey> + CanCache<Req>,
    Req: Request,
{
    Uncached(RefreshCacheFuture<Req, A, C>),
    Outdated(CacheEntry<Req::Result, C::CacheEntryMeta>, RefreshCacheFuture<Req, A, C>),
    // Indirection via Option necessary so we can take the cached entry out of the enum and return it when the future is polled
    UpToDate(Option<CacheEntry<Req::Result, C::CacheEntryMeta>>, Req),
}

impl<Req, A, C> std::fmt::Debug for ProcessRequestFutureState<Req, A, C>
where
    A: ApiClient + MakeRequest<Req>,
    C: Cache + Store<CreatorKey> + Store<NewgroundsSongKey> + CanCache<Req>,
    Req: Request + std::fmt::Debug,
{
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            ProcessRequestFutureState::Uncached(fut) => fmt.debug_tuple("Uncached").field(fut).finish(),
            ProcessRequestFutureState::Outdated(cached, fut) => fmt.debug_tuple("Outdated").field(cached).field(fut).finish(),
            ProcessRequestFutureState::UpToDate(cached, request) => fmt.debug_tuple("UpToDate").field(cached).field(request).finish(),
        }
    }
}

impl<Req, A, C> ProcessRequestFuture<Req, A, C>
where
    A: ApiClient + MakeRequest<Req>,
    C: Cache + CanCache<Req> + CanCache<CreatorKey> + CanCache<NewgroundsSongKey>,
    Req: Request,
{
    pub fn upgrade<Into>(self) -> UpgradeFuture<A, C, Self, Into, Req::Result>
    where
        Req::Result: Upgradable<Into>,
        A: MakeRequest<<Req::Result as Upgradable<Into>>::Request>,
        C: CanCache<<Req::Result as Upgradable<Into>>::Request> + Lookup<<Req::Result as Upgradable<Into>>::LookupKey>,
    {
        UpgradeFuture::new(self.gdcf.clone(), self.forces_refresh, self)
    }
}

// There is no difference between these two methods at this level, but for API consistency we need
// to provide both
impl<Req, A, C> ProcessRequestFuture<Req, A, C>
where
    A: ApiClient + MakeRequest<Req>,
    C: Cache + CanCache<Req> + CanCache<CreatorKey> + CanCache<NewgroundsSongKey>,
    Req: Request,
{
    pub fn upgrade_all<Into>(self) -> UpgradeFuture<A, C, Self, Into, Req::Result>
    where
        Req::Result: Upgradable<Into>,
        A: MakeRequest<<Req::Result as Upgradable<Into>>::Request>,
        C: CanCache<<Req::Result as Upgradable<Into>>::Request> + Lookup<<Req::Result as Upgradable<Into>>::LookupKey>,
    {
        self.upgrade()
    }
}
/*
impl<Req, A, C> CloneCached for ProcessRequestFuture<Req, A, C>
where
    A: ApiClient + MakeRequest<Req>,
    C: Cache + Store<CreatorKey> + Store<NewgroundsSongKey> + CanCache<Req>,
    Req: Request,
    Req::Result: Clone,
{
    fn clone_cached(&self) -> Result<CacheEntry<Self::GdcfItem, <Self::Cache as Cache>::CacheEntryMeta>, ()> {
        match &self.state {
            ProcessRequestFutureState::Exhausted => Err(()),
            ProcessRequestFutureState::Uncached(_) => Ok(CacheEntry::Missing),
            ProcessRequestFutureState::Outdated(cached, _) | ProcessRequestFutureState::UpToDate(cached) => Ok(cached.clone()),
        }
    }
}*/
