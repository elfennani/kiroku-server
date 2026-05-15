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

    pub fn execute(&self, connection: &Connection) -> anyhow::Result<()> {
        connection
            .execute(self.query.as_str(), ())
            .context(format!(
                "Failed to execute migration v{} for query: \"{}\"",
                self.version,
                self.query.trim()
            ))?;

        let mut stmt = connection.prepare("INSERT INTO migrations (version) VALUES (?)")?;
        stmt.execute(params![self.version])?;

        Ok(())
    }
}
