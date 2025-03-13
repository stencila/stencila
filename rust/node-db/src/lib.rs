use kuzu::{Connection, Database, SystemConfig};

use common::eyre::Result;

pub struct NodeDatabase {
    database: Database,
}

impl NodeDatabase {
    pub fn new() -> Result<Self> {
        let database = Database::in_memory(SystemConfig::default())?;

        let instance = Self { database };
        instance.init()?;

        Ok(instance)
    }

    fn init(&self) -> Result<()> {
        self.connection()?.query(include_str!("schema.kuzu"))?;

        Ok(())
    }

    fn connection(&self) -> Result<Connection> {
        Ok(Connection::new(&self.database)?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new() -> Result<()> {
        NodeDatabase::new()?;

        Ok(())
    }
}
