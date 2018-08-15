use core::{backend::Database, statement::Preparation};
use core::backend::Error;
pub use self::insert::{Insert, Insertable};
pub use self::select::Select;
use std::fmt::Debug;

pub mod condition;
pub mod create;
pub mod insert;
pub mod select;
pub mod delete;

pub trait QueryPart<DB: Database>: Debug {
    fn to_sql_unprepared(&self) -> String;

    fn to_sql(&self) -> Preparation<DB> {
        (self.to_sql_unprepared().into(), Vec::new())
    }
}

pub trait Query<DB: Database>: QueryPart<DB> {
    fn execute(&self, db: &DB) -> Result<(), Error<DB>>
        where
            Self: Sized
    {
        db.execute(self)
    }

    fn execute_unprepared(&self, db: &DB) -> Result<(), Error<DB>>
        where
            Self: Sized
    {
        db.execute_unprepared(self)
    }
}

//TODO: DROP TABLE query support