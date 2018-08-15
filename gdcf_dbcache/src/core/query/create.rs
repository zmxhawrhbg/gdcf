use core::backend::Database;
use core::query::Query;
use core::query::QueryPart;
use core::SqlExpr;
use core::table::Field;
use core::types::Type;
use std::marker::PhantomData;

#[derive(Debug)]
pub struct Create<'a, DB: Database + 'a> {
    pub name: &'a str,
    pub ignore_if_exists: bool,
    pub columns: Vec<Column<'a, DB>>,
}

impl<'a, DB: Database + 'a> Create<'a, DB> {
    pub fn new(name: &'a str) -> Create<'a, DB> {
        Create {
            name,
            ignore_if_exists: false,
            columns: Vec::new(),
        }
    }

    pub fn ignore_if_exists(mut self) -> Self {
        self.ignore_if_exists = true;
        self
    }

    pub fn with_column(mut self, col: Column<'a, DB>) -> Self {
        self.columns.push(col);
        self
    }
}

#[derive(Debug)]
pub struct Column<'a, DB: Database + 'a> {
    pub name: &'a str,
    pub sql_type: Box<dyn Type<DB>>,
    pub constraints: Vec<Box<dyn Constraint<DB>>>,
}

impl<'a, DB: Database + 'a> Column<'a, DB> {
    pub fn new<T: Type<DB> + 'static>(name: &'a str, sql_type: T) -> Column<'a, DB> {
        Column {
            name,
            sql_type: Box::new(sql_type),
            constraints: Vec::new(),
        }
    }

    pub fn primary(self) -> Self
        where
            PrimaryKeyConstraint<'a>: Constraint<DB> + 'static
    {
        self.constraint(PrimaryKeyConstraint::default())
    }

    pub fn unique(self) -> Self
        where
            UniqueConstraint<'a>: Constraint<DB> + 'static
    {
        self.constraint(UniqueConstraint::default())
    }

    pub fn not_null(self) -> Self
        where
            NotNullConstraint<'a>: Constraint<DB> + 'static
    {
        self.constraint(NotNullConstraint::default())
    }

    pub fn default<D: SqlExpr<DB>>(self, default: D) -> Self
        where
            DefaultConstraint<'a, DB, D>: Constraint<DB> + 'static
    {
        self.constraint(DefaultConstraint::new(None, default))
    }

    pub fn foreign_key(self, references: &'a Field) -> Self
        where
            ForeignKeyConstraint<'a>: Constraint<DB> + 'static
    {
        self.constraint(ForeignKeyConstraint::new(None, references))
    }

    pub fn constraint<Con: 'static>(mut self, constraint: Con) -> Self
        where
            Con: Constraint<DB>
    {
        self.constraints.push(Box::new(constraint));
        self
    }
}

pub trait Constraint<DB: Database>: QueryPart<DB> {
    fn name(&self) -> Option<&str> {
        None
    }
}

#[derive(Debug, Default)]
pub struct NotNullConstraint<'a>(pub Option<&'a str>);

#[derive(Debug, Default)]
pub struct UniqueConstraint<'a>(pub Option<&'a str>);

#[derive(Debug, Default)]
pub struct PrimaryKeyConstraint<'a>(pub Option<&'a str>);

#[derive(Debug)]
pub struct ForeignKeyConstraint<'a> {
    name: Option<&'a str>,
    references: &'a Field,
}

#[derive(Debug)]
pub struct DefaultConstraint<'a, DB: Database + 'a, D: SqlExpr<DB>> {
    pub name: Option<&'a str>,
    pub default: D,
    _ghost: PhantomData<DB>
}

impl<'a, DB: Database + 'a, D: SqlExpr<DB>> DefaultConstraint<'a, DB, D> {
    pub fn new(name: Option<&'a str>, default: D) -> DefaultConstraint<'a, DB, D> {
        DefaultConstraint {
            name,
            default,
            _ghost: PhantomData
        }
    }
}


impl<'a> ForeignKeyConstraint<'a> {
    pub fn new(name: Option<&'a str>, references: &'a Field) -> ForeignKeyConstraint<'a> {
        ForeignKeyConstraint {
            name,
            references,
        }
    }
}

if_query_part!(NotNullConstraint<'a>, Constraint<DB>);
if_query_part!(UniqueConstraint<'a>, Constraint<DB>);
if_query_part!(PrimaryKeyConstraint<'a>, Constraint<DB>);
if_query_part!(ForeignKeyConstraint<'a>, Constraint<DB>);

impl<'a, DB: Database + 'a> Query<DB> for Create<'a, DB>
    where
        Create<'a, DB>: QueryPart<DB> {}

impl<'a, DB: Database + 'a, D: SqlExpr<DB>> Constraint<DB> for DefaultConstraint<'a, DB, D>
    where
        D: SqlExpr<DB>,
        Self: QueryPart<DB>
{}