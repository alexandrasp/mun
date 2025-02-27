#![cfg(test)]

use crate::{
    db::{AstDatabase, SourceDatabase},
    db::{HirDatabase, Upcast},
    DefDatabase,
};
use mun_target::spec::Target;
use parking_lot::Mutex;

/// A mock implementation of the IR database. It can be used to set up a simple test case.
#[salsa::database(
    crate::SourceDatabaseStorage,
    crate::AstDatabaseStorage,
    crate::InternDatabaseStorage,
    crate::DefDatabaseStorage,
    crate::HirDatabaseStorage
)]
pub(crate) struct MockDatabase {
    storage: salsa::Storage<Self>,
    events: Mutex<Option<Vec<salsa::Event>>>,
}

impl salsa::Database for MockDatabase {
    fn salsa_event(&self, event: salsa::Event) {
        let mut events = self.events.lock();
        if let Some(events) = &mut *events {
            events.push(event);
        }
    }
}

impl Upcast<dyn AstDatabase> for MockDatabase {
    fn upcast(&self) -> &(dyn AstDatabase + 'static) {
        self
    }
}

impl Upcast<dyn DefDatabase> for MockDatabase {
    fn upcast(&self) -> &(dyn DefDatabase + 'static) {
        self
    }
}

impl Upcast<dyn SourceDatabase> for MockDatabase {
    fn upcast(&self) -> &(dyn SourceDatabase + 'static) {
        self
    }
}

impl Default for MockDatabase {
    fn default() -> Self {
        let mut db: MockDatabase = MockDatabase {
            storage: Default::default(),
            events: Default::default(),
        };
        db.set_target(Target::host_target().unwrap());
        db
    }
}

impl MockDatabase {
    pub fn log(&self, f: impl FnOnce()) -> Vec<salsa::Event> {
        *self.events.lock() = Some(Vec::new());
        f();
        self.events.lock().take().unwrap()
    }

    pub fn log_executed(&self, f: impl FnOnce()) -> Vec<String> {
        let events = self.log(f);
        events
            .into_iter()
            .filter_map(|e| match e.kind {
                // This pretty horrible, but `Debug` is the only way to inspect
                // QueryDescriptor at the moment.
                salsa::EventKind::WillExecute { database_key } => {
                    Some(format!("{:?}", database_key.debug(self)))
                }
                _ => None,
            })
            .collect()
    }
}
