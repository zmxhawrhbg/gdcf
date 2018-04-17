use futures::Future;

use tokio_core::reactor::Handle;

use api::request::level::LevelRequest;
use api::GDError;
use model::RawObject;
use api::request::LevelsRequest;

pub trait GDClient
{
    fn handle(&self) -> &Handle;

    fn level(&self, req: LevelRequest) -> Box<Future<Item=RawObject, Error=GDError> + 'static>;
    fn levels(&self, req: LevelsRequest) -> Box<Future<Item=Vec<RawObject>, Error=GDError> + 'static>;
}