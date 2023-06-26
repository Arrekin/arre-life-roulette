use chrono::Utc;
use rusqlite::{Connection, Result, Row};
use crate::errors::{ArreError, ArreResult};
use crate::utils::{ArreDateTime, Id};

pub fn tag_persist(conn: &Connection, tag: &mut Tag) -> ArreResult<()> {
    let dt = ArreDateTime::now();
    conn.execute("
        INSERT INTO tags (created_date, updated_date, name, color) VALUES (?1, ?2, ?3, ?4);
        ", (dt.clone(), dt.clone(), &tag.name, &tag.color),
    )?;
    let mut stmt = conn.prepare("
        SELECT
         tag_id, created_date, updated_date, name, color
        FROM tags
        WHERE tag_id = last_insert_rowid()
    ")?;
    Ok(stmt.query_row([], |row| {
        tag.update_from_row(row)
    })?)
}

pub fn tag_get_all<C>(conn: &Connection) -> ArreResult<C>
where C: FromIterator<Tag>
{
    let mut stmt = conn.prepare("
        SELECT
         tag_id, created_date, updated_date, name, color
        FROM tags
    ")?;
    let result = stmt.query_map([], |row| {
        Tag::from_row(row)
    })?.collect::<Result<C>>()?;
    Ok(result)
}

pub fn tag_update(conn: &Connection, tag: &Tag) -> ArreResult<()> {
    conn.execute("
        UPDATE tags
        SET
         updated_date = ?1, name = ?2, color = ?3
        WHERE tag_id = ?4
    ", (Utc::now().to_string(), &tag.name, &tag.color, tag.get_id()?),
    )?;
    Ok(())
}

pub fn tag_delete(conn: &Connection, id: impl Into<TagId>) -> ArreResult<()> {
    let id = id.into();
    conn.execute("DELETE FROM tags WHERE tag_id = ?1;", (id,))?;
    Ok(())
}

pub type TagId = Id<Tag>;
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Tag {
    pub id: Option<TagId>,
    pub created_date: ArreDateTime<Utc>,
    pub updated_date: ArreDateTime<Utc>,
    pub name: String,
    pub color: String,
}

impl Tag {
    pub fn from_row(row: &Row) -> Result<Tag> {
        Ok(Tag {
            id: Some(row.get(0)?),
            created_date: row.get(1)?,
            updated_date: row.get(2)?,
            name: row.get(3)?,
            color: row.get(4)?,
        })
    }

    pub fn update_from_row(&mut self, row: &Row) -> Result<()> {
        self.id = Some(row.get(0)?);
        self.created_date = row.get(1)?;
        self.updated_date = row.get(2)?;
        self.name = row.get(3)?;
        self.color = row.get(4)?;
        Ok(())
    }

    pub fn get_id(&self) -> ArreResult<TagId> {
        self.id.ok_or(ArreError::ItemNotPersisted().into())
    }
}

impl Default for Tag {
    fn default() -> Self {
        Self {
            id: None,
            created_date: Utc::now().into(),
            updated_date: Utc::now().into(),
            name: String::new(),
            color: "#00FFFF".to_string(),
        }
    }
}


#[cfg(test)]
mod tests {
    use rstest::*;
    use rusqlite::Connection;
    use crate::test_fixtures::{conn, TestFactory};
    use super::*;

    #[rstest]
    fn tag_persist_successful(conn: Connection) -> ArreResult<()> {
        let tf = TestFactory::new(&conn);
        let mut tag = Tag::default();
        match tag.get_id() {
            Ok(_) => { panic!("TagId should not be available in non persisted tag")}
            Err(err) => {
                if let Some(&ArreError::ItemNotPersisted()) = err.downcast_ref::<ArreError>() {
                    // The expected outcome. Continue.
                } else {  panic!("Unexpected error: {:?}", err) }
            }
        }
        tag_persist(&conn, &mut tag)?;
        tf.assert_tag_exist(tag.get_id()?, true)
    }


    #[rstest]
    fn tag_update_successful(conn: Connection) -> ArreResult<()> {
        let mut tf = TestFactory::new(&conn);
        let mut tag = tf.create_tags(1)?.pop().unwrap();
        tag.name = "Glorious Tag".to_string();
        tag.color = "#00FFFF".to_string();
        tag_update(&conn, &tag)?;
        let tag = &tag_get_all::<Vec<_>>(&conn)?[0];
        assert_eq!(
            tag.name, "Glorious Tag",
            "Tag name is wrong. Expected Glorious Tag, got {:?}",
            tag.name
        );
        assert_eq!(
            tag.color, "#00FFFF",
            "Tag color is wrong. Expected #00FFFF, got {:?}",
            tag.color
        );
        Ok(())
    }

    #[rstest]
    #[case(0)]
    #[case(1)]
    #[case(5)]
    fn tag_get_all_successful(
        conn: Connection,
        #[case] expected_number_of_tags: usize,
    ) -> ArreResult<()> {
        let mut tf = TestFactory::new(&conn);
        // Create tags and check that get_all() returns all of them
        tf.create_tags(expected_number_of_tags)?;
        let tags = tag_get_all::<Vec<_>>(&conn)?;
        assert_eq!(tags.len(), expected_number_of_tags);
        Ok(())
    }

    #[rstest]
    fn tag_delete_successful(conn: Connection) -> ArreResult<()> {
        let mut tf = TestFactory::new(&conn);
        let tags_nb = 3;
        let tags = tf.create_tags(tags_nb)?;
        tf.assert_table_count("tags", tags_nb)?;
        for idx in (0..tags_nb).rev() {
            tag_delete(&conn, tags[idx].get_id()?)?;
            tf.assert_table_count("tags", idx)?;
        }
        Ok(())
    }
}
