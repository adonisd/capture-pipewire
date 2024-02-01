use ashpd::desktop::screencast::Stream as ScreencastStream;
use ashpd::{
    desktop::screencast::{CursorMode, PersistMode, Screencast, SourceType},
    WindowIdentifier,
};
mod pipewire_modules; // Import the initialize_pipewire function from the pipewire module

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

    let stream = response.streams().first().unwrap().clone();
    Ok(stream)
}

#[tokio::main]
async fn main() -> ashpd::Result<()> {
    let screen_cast_stream = create_screencast_stream()
        .await
        .expect("Failed to create screencast stream");
    println!("{:#?}", screen_cast_stream);

    let (stream, data, mainloop) = pipewire_modules::initialize_pipewire();
    pipewire_modules::handle_buffers(&stream, data, false);
    println!("Created stream {:#?}", stream);
    pipewire_modules::connect_stream(&stream, screen_cast_stream.pipe_wire_node_id());
    eprintln!("Connected stream");
    mainloop.run();
    Ok(())
}
