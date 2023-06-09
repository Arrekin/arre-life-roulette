use std::fmt::{Debug, Display, Formatter};
use std::hash::{Hash, Hasher};
use std::marker::PhantomData;
use std::ops::Deref;
use std::str::FromStr;
use chrono::{DateTime, TimeZone};
use rusqlite::types::{FromSql, FromSqlResult, ToSql};

#[derive(Debug, Eq, PartialEq)]
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
impl<T> Into<i64> for Id<T> {
    fn into(self) -> i64 {
        self.id
    }
}

impl<T> ToSql for Id<T> {
    fn to_sql(&self) -> rusqlite::Result<rusqlite::types::ToSqlOutput> {
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

impl <T>Hash for Id<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

impl <T>Display for Id<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.id)
    }
}

#[derive(Debug, Eq, PartialEq, Hash, Clone)]
pub struct ArreDateTime<Tz: TimeZone> {
    date_time: DateTime<Tz>,
}

impl<Tz: TimeZone> ArreDateTime<Tz> {
    pub fn new(date_time: DateTime<Tz>) -> Self {
        ArreDateTime { date_time}
    }
}

impl<Tz: TimeZone> FromSql for ArreDateTime<Tz>
where DateTime<Tz>: FromStr
{
    fn column_result(value: rusqlite::types::ValueRef) -> FromSqlResult<Self> {
        let date_time = String::column_result(value)?
            .parse::<DateTime<Tz>>()
            .unwrap_or_else(|_| panic!("Invalid date format in the DB"));
        Ok(ArreDateTime { date_time })
    }
}

impl<Tz: TimeZone> From<DateTime<Tz>> for ArreDateTime<Tz> {
    fn from(date_time: DateTime<Tz>) -> Self {
        ArreDateTime { date_time }
    }
}

impl<Tz: TimeZone> ToSql for ArreDateTime<Tz>
where DateTime<Tz>: Display
{
    fn to_sql(&self) -> rusqlite::Result<rusqlite::types::ToSqlOutput> {
        let str_dt = self.date_time.to_string();
        Ok(rusqlite::types::ToSqlOutput::Owned(str_dt.into()))
    }
}

impl <Tz: TimeZone> Deref for ArreDateTime<Tz> {
    type Target = DateTime<Tz>;

    fn deref(&self) -> &Self::Target {
        &self.date_time
    }
}

