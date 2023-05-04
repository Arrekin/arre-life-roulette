use rusqlite::{Connection, Result, Row};
use crate::errors::{ArreResult};
use crate::item::ItemId;

pub fn item_stats_update(conn: &Connection, stats: &ItemStats) -> ArreResult<()> {
    conn.execute("
        UPDATE item_stats
        SET times_worked = ?1, time_spent = ?2
        WHERE item_id = ?3
    ", (stats.times_worked, stats.time_spent, stats.id),
    )?;
    Ok(())
}

pub fn item_stats_get(conn: &Connection, id: impl Into<ItemId>) -> ArreResult<ItemStats> {
    let mut stmt = conn.prepare("
        SELECT
         item_id, times_worked, time_spent
        FROM item_stats
        WHERE item_id = ?1
    ")?;
    Ok(stmt.query_row([id.into()], |row| {
        ItemStats::from_row(row)
    })?)
}


#[derive(Debug, Clone, Eq, PartialEq)]
pub struct ItemStats {
    pub id: ItemId,
    pub times_worked: usize,
    pub time_spent: usize,
}

impl ItemStats {
    pub fn from_row(row: &Row) -> Result<ItemStats> {
        Ok(ItemStats {
            id: row.get(0)?,
            times_worked: row.get(1)?,
            time_spent: row.get(2)?,
        })
    }
}

// TODO: I don't like having default for ItemStats, think of other way
impl Default for ItemStats {
    fn default() -> Self {
        ItemStats {
            id: 0.into(),
            times_worked: 0,
            time_spent: 0,
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
    fn item_creates_removes_stats(conn: Connection) -> ArreResult<()> {
        let tf = TestFactory::new(&conn);
        let item = item_create(&conn, "Name", "Description")?;
        let item_id = item.get_id()?;
        let stats = item_stats_get(&conn, item_id)?;
        assert_eq!( stats.times_worked, 0, "times_worked of fresh Item should be 0");
        assert_eq!( stats.time_spent, 0, "time_spent of fresh Item should be 0");

        // Delete the item and check that stats were deleted as well
        item_delete(&conn, item_id)?;
        tf.assert_item_exist(item_id, false)?;
        tf.assert_items_number(0)?;
        assert!(item_stats_get(&conn, item_id).is_err(), "Item stats should have been deleted");
        Ok(())
    }

    #[rstest]
    fn item_persist_creates_stats(conn: Connection) -> ArreResult<()> {
        let mut item = Item::default();
        item_persist(&conn, &mut item)?;
        assert!(item.get_id().is_ok(), "Item id should appear after persist");

        let stats = item_stats_get(&conn, item.get_id()?)?;
        assert_eq!(stats.times_worked, 0, "times_worked of persisted Item should be 0");
        assert_eq!(stats.time_spent, 0, "time_spent of persisted Item should be 0");
        Ok(())
    }

    #[rstest]
    fn update_item_stats(
        conn: Connection,
    ) -> ArreResult<()> {
        let item = item_create(&conn, "Name", "Description")?;
        let mut stats = item_stats_get(&conn, item.get_id()?)?;
        stats.times_worked = 5;
        stats.time_spent = 10;
        item_stats_update(&conn, &stats)?;
        let stats = item_stats_get(&conn, item.get_id()?)?;
        assert_eq!(stats.times_worked, 5);
        assert_eq!(stats.time_spent, 10);
        Ok(())
    }
}
