use std::{
    env,
    os::unix::net::UnixStream,
    path::{self, PathBuf},
};

use wayland_client::Connection;

pub fn create_connection(socket_path: Option<PathBuf>) -> Connection {
    let path_to_wayland_socket = match socket_path {
        Some(path_to_wayland_socket) => path_to_wayland_socket,
        None => {
            let xdg_runtime_dir = env::var("XDG_RUNTIME_DIR").unwrap();
            let wayland_display = env::var("WAYLAND_DISPLAY").unwrap();

            let mut path_to_wayland_socket = PathBuf::from(xdg_runtime_dir);
            path_to_wayland_socket.push(wayland_display);
            path_to_wayland_socket
        }
    };

    let stream = UnixStream::connect(path_to_wayland_socket).unwrap();
    Connection::from_socket(stream).unwrap()
}

#[cfg(test)]
mod test {
    use wayland_client::{
        backend::ObjectData,
        globals::Global,
        protocol::{wl_output::WlOutput, wl_registry::WlRegistry},
        Dispatch, EventQueue, Proxy,
    };

    use crate::create_connection;
    struct State {
        globals: Vec<Global>,
    }
    impl Dispatch<WlRegistry, ()> for State {
        fn event(
            state: &mut Self,
            proxy: &WlRegistry,
            event: <WlRegistry as Proxy>::Event,
            data: &(),
            conn: &wayland_client::Connection,
            qhandle: &wayland_client::QueueHandle<Self>,
        ) {
            match event {
                wayland_client::protocol::wl_registry::Event::Global {
                    name,
                    interface,
                    version,
                } => {
                    state.globals.push(Global {
                        name,
                        interface,
                        version,
                    });
                }
                wayland_client::protocol::wl_registry::Event::GlobalRemove { name } => {
                    let position = state
                        .globals
                        .iter()
                        .position(|global| global.name.eq(&name));
                    if let Some(position) = position {
                        state.globals.remove(position);
                    };
                }
                _ => (),
            }
        }
    }
    impl Dispatch<WlOutput, i32> for State {
        fn event(
            state: &mut Self,
            proxy: &WlOutput,
            event: <WlOutput as Proxy>::Event,
            data: &i32,
            conn: &wayland_client::Connection,
            qhandle: &wayland_client::QueueHandle<Self>,
        ) {
            match event {
                wayland_client::protocol::wl_output::Event::Done => {
                    println!("A done event happened")
                }
                wayland_client::protocol::wl_output::Event::Mode {
                    flags,
                    width,
                    height,
                    refresh,
                } => {
                    println!("A mode event happened")
                }
                wayland_client::protocol::wl_output::Event::Geometry {
                    x,
                    y,
                    physical_width,
                    physical_height,
                    subpixel,
                    make,
                    model,
                    transform,
                } => {
                    println!("A geometry event happened")
                }
                _ => {
                    println!("Some other event happened")
                }
            };
        }
    }

    #[test]
    fn test_create_connection() {
        let connection = create_connection(None);
        let display = connection.display();
        assert!(display.is_alive());
        let mut event_queue = connection.new_event_queue::<State>();
        let qh = event_queue.handle();
        let mut state = State { globals: vec![] };
        let registry = display.get_registry(&qh, ());

        event_queue
            .roundtrip(&mut state)
            .expect("Event queue roundtrip should not fail");

        dbg!(&state.globals);
        assert!(!state.globals.is_empty());

        let first_output = state
            .globals
            .iter()
            .find(|global| global.interface == "wl_output" && global.version == 4)
            .expect("No version 4 wl_outputs found.");

        let output: WlOutput = registry.bind(first_output.name, first_output.version, &qh, 3);
        let data: &i32 = output.data().unwrap();
        assert_eq!(*data, 2);

        event_queue
            .roundtrip(&mut state)
            .expect("Event queue roundtrip should not fail");
    }
}
