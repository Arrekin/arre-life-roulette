use chrono::{Duration};
use rusqlite::{Connection, Result, Row};
use crate::errors::{ArreResult};
use crate::item::ItemId;

pub fn item_details_update(conn: &Connection, stats: &ItemDetails) -> ArreResult<()> {
    conn.execute("
        UPDATE item_details
        SET session_duration = ?2
        WHERE item_id = ?1
    ", (stats.id, stats.session_duration.map(|sd| sd.num_seconds())),
    )?;
    Ok(())
}

pub fn item_details_get(conn: &Connection, id: impl Into<ItemId>) -> ArreResult<ItemDetails> {
    let mut stmt = conn.prepare("
        SELECT
         item_id, session_duration
        FROM item_details
        WHERE item_id = ?1
    ")?;
    Ok(stmt.query_row([id.into()], |row| {
        ItemDetails::from_row(row)
    })?)
}


#[derive(Debug, Clone, Eq, PartialEq)]
pub struct ItemDetails {
    pub id: ItemId,
    pub session_duration: Option<Duration>, // in seconds
}

impl ItemDetails {
    pub fn from_row(row: &Row) -> Result<ItemDetails> {
        Ok(ItemDetails {
            id: row.get(0)?,
            session_duration: row.get::<_, Option<i64>>(1)?.map(Duration::seconds),
        })
    }
}

impl Default for ItemDetails {
    fn default() -> Self {
        ItemDetails {
            id: 0.into(),
            session_duration: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use rstest::*;
    use rusqlite::Connection;
    use crate::item::{Item, item_create, item_delete, item_persist};
    use crate::test_fixtures::{conn, TestFactory};
    use super::*;

    #[rstest]
    fn item_creates_removes_details(conn: Connection) -> ArreResult<()> {
        let tf = TestFactory::new(&conn);
        let item_id = item_create(&conn, "Name", "Description")?.get_id()?;
        let details = item_details_get(&conn, item_id)?;
        assert_eq!( details.session_duration, None, "default session_duration should be None");

        // Delete the item and check that details were deleted as well
        item_delete(&conn, item_id)?;
        tf.assert_item_exist(item_id, false)?;
        tf.assert_table_count("item_details", 0)?;
        assert!(item_details_get(&conn, item_id).is_err(), "Item details should have been deleted");
        Ok(())
    }

    #[rstest]
    fn item_persist_creates_details(conn: Connection) -> ArreResult<()> {
        let mut item = Item::default();
        item_persist(&conn, &mut item)?;
        assert!(item.get_id().is_ok(), "Item id should appear after persist");

        let details = item_details_get(&conn, item.get_id()?)?;
        assert_eq!(details.session_duration, None, "default session_duration should be None");
        Ok(())
    }

    #[rstest]
    #[case(None)]
    #[case(Some(Duration::seconds(10)))]
    fn update_item_details(
        conn: Connection,
        #[case] session_duration: Option<Duration>,
    ) -> ArreResult<()> {
        let item_id = item_create(&conn, "Name", "Description")?.get_id()?;
        let mut details = item_details_get(&conn, item_id)?;
        details.session_duration = session_duration;
        item_details_update(&conn, &details)?;
        let details = item_details_get(&conn, item_id)?;
        assert_eq!(details.session_duration, session_duration);
        Ok(())
    }
}
