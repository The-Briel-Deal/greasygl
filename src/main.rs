use std::{
    os::fd::OwnedFd,
    sync::{Arc, Mutex},
};

use wayland_client::{
    backend::{protocol::Message, ObjectData, ObjectId},
    globals::Global,
    protocol::{wl_display, wl_registry::WlRegistry},
    Connection, Proxy,
};

#[derive(Debug)]
struct GreasyRegistryInner {
    globals: Vec<Global>,
}
#[derive(Debug)]
struct GreasyRegistry {
    connection: Arc<Connection>,
    state: Mutex<GreasyRegistryInner>,
}
impl GreasyRegistry {
    fn new(connection: Arc<Connection>) -> Self {
        GreasyRegistry {
            connection,
            state: Mutex::new(GreasyRegistryInner { globals: vec![] }),
        }
    }
}

impl ObjectData for GreasyRegistry {
    fn event(
        self: std::sync::Arc<Self>,
        _backend: &wayland_client::backend::Backend,
        msg: Message<ObjectId, OwnedFd>,
    ) -> Option<std::sync::Arc<dyn ObjectData>> {
        let (_registry, event) = WlRegistry::parse_event(&self.connection, msg).unwrap();

        match event {
            wayland_client::protocol::wl_registry::Event::Global {
                name,
                interface,
                version,
            } => {
                let mut state = self.state.lock().unwrap();
                state.globals.push(Global {
                    name,
                    interface,
                    version,
                });
            }
            wayland_client::protocol::wl_registry::Event::GlobalRemove { name } => {
                let mut state = self.state.lock().unwrap();
                let index = state
                    .globals
                    .iter()
                    .position(|global| global.name == name)
                    .unwrap();
                state.globals.remove(index);
            }
            _ => {}
        };

        None
    }
    fn destroyed(&self, _object_id: wayland_client::backend::ObjectId) {
        unreachable!();
    }
}

fn main() {
    let connection = Arc::new(Connection::connect_to_env().unwrap());
    let display = connection.display();
    let registry_data = Arc::new(GreasyRegistry::new(Arc::clone(&connection)));
    let registry: WlRegistry = display
        .send_constructor(
            wl_display::Request::GetRegistry {},
            registry_data,
        )
        .unwrap();
    connection.roundtrip().unwrap();
}
