use std::fs::{
    canonicalize,
    read,
};

use anyhow::Result;
use futures::{
    select,
    FutureExt,
};
use simplydmx::{
    api_utilities::{
        spawn_api_facet_controller,
        JSONCommand,
        JSONResponse,
    },
    async_std::{
        channel::{
            self,
            Sender,
        },
        task::{
            block_on,
            self,
        },
    },
    async_main,
    PluginManager,
};

use wry::{
    application::{
        event_loop::{
            ControlFlow,
            EventLoop,
            EventLoopWindowTarget,
            EventLoopProxy,
        },
        window::WindowBuilder, event::{
            Event,
            StartCause,
        },
    },
    http::ResponseBuilder,
    webview::{
        WebViewBuilder,
        WebView,
    },
};

#[cfg(target_os = "android")]
use wry::android_binding;

#[cfg(target_os = "android")]
fn init_logging() {
    android_logger::init_once(
        android_logger::Config::default()
            .with_min_level(log::Level::Trace)
            .with_tag("tauri-test"),
    );
}

#[cfg(not(target_os = "android"))]
fn init_logging() {
    env_logger::init();
}

fn stop_unwind<F: FnOnce() -> T, T>(f: F) -> T {
    match std::panic::catch_unwind(std::panic::AssertUnwindSafe(f)) {
        Ok(t) => t,
        Err(err) => {
            eprintln!("attempt to unwind out of `rust` with err: {:?}", err);
            std::process::abort()
        }
    }
}

fn _start_app() {
    stop_unwind(|| main().unwrap());
}

#[no_mangle]
#[inline(never)]
pub extern "C" fn start_app() {
    #[cfg(target_os = "android")]
    android_binding!(com_shaunkeys, tauri_1test, _start_app);
    _start_app()
}

fn start_simplydmx(event_loop_proxy: EventLoopProxy<String>) -> Sender<String> {
// fn start_simplydmx(event_loop_proxy: EventLoopProxy<String>) {
    let manager = PluginManager::new();
    let plugin = block_on(manager.register_plugin("gui", "iPad Wry UI")).unwrap();

    block_on(async_main(manager.clone(), None));

    let (request_sender, request_receiver) = channel::unbounded::<JSONCommand>();
    let (response_sender, response_receiver) = channel::unbounded::<JSONResponse>();
    let plugin_clone = plugin.clone();
    block_on(spawn_api_facet_controller(plugin_clone, request_receiver, response_sender));

    let (ui_sender, ui_receiver) = channel::unbounded::<String>();

    task::spawn(async move {
        loop {
            select! {
                msg = ui_receiver.recv().fuse() => {
                    if let Ok(ref msg) = msg {
                        let command: JSONCommand = serde_json::from_str(msg).unwrap();
                        request_sender.send(command).await.unwrap();
                    } else {
                        break;
                    }
                },
                msg = response_receiver.recv().fuse() => {
                    if let Ok(msg) = msg {
                        event_loop_proxy.send_event(serde_json::to_string(&msg).unwrap()).unwrap();
                    }
                },
            }
        }
    });

    return ui_sender;
}

fn main() -> Result<()> {
    init_logging();
    let event_loop = EventLoop::<String>::with_user_event();

    let event_loop_proxy = event_loop.create_proxy();
    let sender = start_simplydmx(event_loop_proxy);

    let mut webview = None;
    event_loop.run(move |event, event_loop, control_flow| {
        *control_flow = ControlFlow::Wait;
        println!("{:?}", event);

        match event {
            Event::NewEvents(StartCause::Init) => {
                webview = Some(build_webview(event_loop, sender.clone()).unwrap());
                // webview = Some(build_webview(event_loop).unwrap());
            },
            Event::UserEvent(message) => {
                webview.as_ref().unwrap().evaluate_script(&format!("__rust_response__({});", message)).unwrap();
            },
            _ => (),
        }
    });
}

fn build_webview(event_loop: &EventLoopWindowTarget<String>, command_sender: Sender<String>) -> Result<WebView> {
// fn build_webview(event_loop: &EventLoopWindowTarget<String>) -> Result<WebView> {
    let window = WindowBuilder::new()
        .with_title("A fantastic window!")
        .build(&event_loop)?;
    let webview = WebViewBuilder::new(window)?
        .with_url("wry://assets/index.html")?
        // If you want to use custom protocol, set url like this and add files like index.html to assets directory.
        // .with_url("wry://assets/index.html")?
        .with_devtools(true)
        // .with_initialization_script("console.log('hello world from init script');")
        .with_ipc_handler(move |_, s| {
            dbg!(&s);
            block_on(command_sender.send(s)).unwrap();
        })
        .with_custom_protocol("wry".into(), move |request| {
            #[cfg(not(target_os = "android"))]
            {
                // Remove url scheme
                let path = request.uri().replace("wry://", "");
                let mut resource = core_foundation::bundle::CFBundle::main_bundle()
                    .resources_path()
                    .unwrap();
                resource.push(&path);
                // Read the file content from file path
                let content = read(canonicalize(&resource)?)?;

                // Return asset contents and mime types based on file extentions
                // If you don't want to do this manually, there are some crates for you.
                // Such as `infer` and `mime_guess`.
                let (data, meta) = if path.ends_with(".html") {
                    (content, "text/html")
                } else if path.ends_with(".js") {
                    (content, "text/javascript")
                } else if path.ends_with(".png") {
                    (content, "image/png")
                } else if path.ends_with(".css") {
                    (content, "text/css")
                } else if path.ends_with(".svg") {
                    (content, "image/svg+xml")
                } else {
                    unimplemented!();
                };

                ResponseBuilder::new().mimetype(meta).body(data)
            }

            #[cfg(target_os = "android")]
            {
                ResponseBuilder::new().body(vec![])
            }
        })
        .build()?;

        Ok(webview)
 }

