use ashpd::desktop::screencast::Stream as ScreencastStream;
use ashpd::{
    desktop::screencast::{CursorMode, PersistMode, Screencast, SourceType},
    WindowIdentifier,
};
mod pipewire_modules;

async fn create_screencast_stream() -> ashpd::Result<ScreencastStream> {
    let proxy = Screencast::new()
        .await
        .expect("Failed to create screencast proxy");
    let session = proxy
        .create_session()
        .await
        .expect("Failed to create session");
    proxy
        .select_sources(
            &session,
            CursorMode::Metadata,
            SourceType::Monitor | SourceType::Window,
            true,
            None,
            PersistMode::ExplicitlyRevoked,
        )
        .await
        .expect("Failed to select sources");
    let response = proxy
        .start(&session, &WindowIdentifier::default())
        .await
        .expect("Failed to start screencast")
        .response()
        .expect("Failed to get response from screencast");

    let screencast_stream = response.streams().first().unwrap().clone();
    Ok(screencast_stream)
}

#[tokio::main]
async fn main() -> ashpd::Result<()> {
    let screen_cast_stream = create_screencast_stream()
        .await
        .expect("Failed to create screencast stream");
    println!("{:#?}", screen_cast_stream);
    let id = screen_cast_stream.pipe_wire_node_id();

    pipewire::init();

    let mainloop = pipewire::MainLoop::new().expect("Failed to create mainloop");
    let context = pipewire::Context::new(&mainloop).expect("Failed to create context");
    let core = context
        .connect(None)
        .expect("Failed to connect to PipeWire");

    let stream =
        pipewire_modules::initialize_pipewire(core).expect("Failed to initialize pipewire");
    println!("Created stream {:#?}", stream);
    let _listener =
        pipewire_modules::handle_buffers(&stream, false).expect("Failed to handle buffers");
    println!("Connecting stream");
    pipewire_modules::connect_stream(&stream, id);
    eprintln!("Connected stream");
    mainloop.run();
    Ok(())
}
