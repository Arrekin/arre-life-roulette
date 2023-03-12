use std::marker::PhantomData;
use std::ops::Deref;
use rusqlite::{Result};
use rusqlite::types::{ToSql, FromSql, FromSqlResult};

#[derive(Debug, PartialEq)]
pub struct Id<T> {
    id: i64,
    phantom: PhantomData<T>,
}

impl<T> Id<T> {
    pub fn new(id: i64) -> Self {
        Id { id, phantom: PhantomData }
    }
}

impl<T> FromSql for Id<T> {
    fn column_result(value: rusqlite::types::ValueRef) -> FromSqlResult<Self> {
        let id = i64::column_result(value)?;
        Ok(Id { id, phantom: PhantomData })
    }
}

impl<T> From<i64> for Id<T> {
    fn from(value: i64) -> Self {
        Id { id: value, phantom: PhantomData }
    }
}

impl<T> ToSql for Id<T> {
    fn to_sql(&self) -> Result<rusqlite::types::ToSqlOutput> {
        self.id.to_sql()
    }
}

impl <T> Clone for Id<T> {
    fn clone(&self) -> Id<T> {
        Id { id: self.id, phantom: PhantomData }
    }
}
impl <T> Copy for Id<T> {}

impl <T> Deref for Id<T> {
    type Target = i64;

    fn deref(&self) -> &Self::Target {
        &self.id
    }
}