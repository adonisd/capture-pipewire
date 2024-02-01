use std::io::{stdout, Write};

pub struct UserData {
    pub(crate) format: pipewire::spa::param::video::VideoInfoRaw,
}

pub fn initialize_pipewire() -> (pipewire::stream::Stream, UserData, pipewire::MainLoop) {
    pipewire::init();

    let mainloop = pipewire::MainLoop::new().expect("Failed to create mainloop");
    let context = pipewire::Context::new(&mainloop).expect("Failed to create context");
    let core = context
        .connect(None)
        .expect("Failed to connect to PipeWire");

    let data = UserData {
        format: Default::default(),
    };

    let stream = pipewire::stream::Stream::new(
        &core,
        "captured-stream",
        pipewire::properties! {
            *pipewire::keys::MEDIA_TYPE => "Video",
            *pipewire::keys::MEDIA_CATEGORY => "Capture",
            *pipewire::keys::MEDIA_ROLE => "Screen",
        },
    )
    .expect("Failed to create stream");
    (stream, data, mainloop)
}

pub fn handle_buffers(stream: &pipewire::stream::Stream, data: UserData, check: bool) {
    let _listener = stream
        .add_local_listener_with_user_data(data)
        .state_changed(|old, new| {
            eprintln!("State changed: {:?} -> {:?}", old, new);
        })
        .param_changed(move |_, id, user_data, pod_options| {
            let Some(options) = pod_options else {
                return;
            };
            if id != pipewire::spa::param::ParamType::Format.as_raw() {
                return;
            }

            let (media_type, media_subtype) =
                match pipewire::spa::param::format_utils::parse_format(options) {
                    Ok(v) => v,
                    Err(_) => return,
                };

            if media_type != pipewire::spa::format::MediaType::Video
                || media_subtype != pipewire::spa::format::MediaSubtype::Raw
            {
                return;
            }

            user_data
                .format
                .parse(options)
                .expect("Failed to parse param changed to VideoInfoRaw");

            eprintln!("got video format:");
            eprintln!(
                "  format: {} ({:?})",
                user_data.format.format().as_raw(),
                user_data.format.format()
            );
            eprintln!(
                "  size: {}x{}",
                user_data.format.size().width,
                user_data.format.size().height
            );
            eprintln!(
                "  framerate: {}/{}",
                user_data.format.framerate().num,
                user_data.format.framerate().denom
            );
            if check == true {
                std::process::exit(0)
            }
        })
        .process(|stream, _| {
            match stream.dequeue_buffer() {
                None => eprintln!("out of buffers"),
                Some(mut buffer) => {
                    let datas = buffer.datas_mut();
                    if datas.is_empty() {
                        return;
                    }
                    let data = &mut datas[0];
                    let buff: Vec<u8> = data.data().unwrap().to_vec();
                    stdout().write_all(&buff).unwrap(); //raw frames to stdout
                }
            }
        })
        .register()
        .expect("Failed to register listener");
}

pub fn connect_stream(stream: &pipewire::stream::Stream, id: u32) {
    let obj = pipewire::spa::pod::object!(
        pipewire::spa::utils::SpaTypes::ObjectParamFormat,
        pipewire::spa::param::ParamType::EnumFormat,
        pipewire::spa::pod::property!(
            pipewire::spa::format::FormatProperties::MediaType,
            Id,
            pipewire::spa::format::MediaType::Video
        ),
        pipewire::spa::pod::property!(
            pipewire::spa::format::FormatProperties::MediaSubtype,
            Id,
            pipewire::spa::format::MediaSubtype::Raw
        ),
        pipewire::spa::pod::property!(
            pipewire::spa::format::FormatProperties::VideoFormat,
            Choice,
            Enum,
            Id,
            pipewire::spa::param::video::VideoFormat::RGB,
            pipewire::spa::param::video::VideoFormat::RGBA,
            pipewire::spa::param::video::VideoFormat::RGBx,
            pipewire::spa::param::video::VideoFormat::BGRx,
            pipewire::spa::param::video::VideoFormat::YUY2,
            pipewire::spa::param::video::VideoFormat::I420,
        ),
        pipewire::spa::pod::property!(
            pipewire::spa::format::FormatProperties::VideoSize,
            Choice,
            Range,
            Rectangle,
            pipewire::spa::utils::Rectangle {
                width: 320,
                height: 240
            },
            pipewire::spa::utils::Rectangle {
                width: 1,
                height: 1
            },
            pipewire::spa::utils::Rectangle {
                width: 4096,
                height: 4096
            }
        ),
        pipewire::spa::pod::property!(
            pipewire::spa::format::FormatProperties::VideoFramerate,
            Choice,
            Range,
            Fraction,
            pipewire::spa::utils::Fraction { num: 25, denom: 1 },
            pipewire::spa::utils::Fraction { num: 0, denom: 1 },
            pipewire::spa::utils::Fraction {
                num: 1000,
                denom: 1
            }
        ),
    );
    let values: Vec<u8> = pipewire::spa::pod::serialize::PodSerializer::serialize(
        std::io::Cursor::new(Vec::new()),
        &pipewire::spa::pod::Value::Object(obj),
    )
    .unwrap()
    .0
    .into_inner();

    let mut params = [pipewire::spa::pod::Pod::from_bytes(&values).unwrap()];

    stream
        .connect(
            pipewire::spa::Direction::Input,
            Some(id),
            pipewire::stream::StreamFlags::AUTOCONNECT | pipewire::stream::StreamFlags::MAP_BUFFERS,
            &mut params,
        )
        .expect("Failed to connect to established stream");
}
