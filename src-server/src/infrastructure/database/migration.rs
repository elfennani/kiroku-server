use anyhow::Context;
use rusqlite::{Connection, params};

pub struct Migration {
    pub version: i32,
    query: String,
}

impl Migration {
    pub fn new(version: i32, query: &str) -> anyhow::Result<Self> {
        if version <= 0 {
            return Err(anyhow::anyhow!("Version must be greater than 0"));
        }

        Ok(Self {
            version,
            query: query.to_owned(),
        })
    }

    pub fn execute(&self, connection: &mut Connection) -> anyhow::Result<()> {
        let tx = connection.transaction()?;
        let queries = self.query.split(";").into_iter();

        for query in queries {
            if query.trim().is_empty() { continue; }
            
            tx.execute(query, ()).context(format!(
                "Failed to execute migration v{} for query: \"{}\"",
                self.version,
                query
            ))?;
        }

        {
            let mut stmt = tx.prepare("INSERT INTO migrations (version) VALUES (?)")?;
            stmt.execute(params![self.version])?;
        }

        tx.commit().context(format!(
            "Failed to commit migration version {}",
            self.version
        ))?;

        Ok(())
    }
}
