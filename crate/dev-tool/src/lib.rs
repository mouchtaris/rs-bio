use {
    bio::*,
    std::{
        borrow::BorrowMut,
        ffi::OsStr,
        fmt,
        fs,
        io,
        process::Command,
    },
};

pub fn fmt() -> IO<()> {
    run_cmd(["cargo", "+nightly", "fmt"])?;

    Ok(())
}

pub fn doc() -> IO<()> {
    run_cmd(["cargo", "doc"])?;

    const CSS: &str = "target/doc/rustdoc.css";
    let css = fs::read_to_string(CSS)?;
    let css = css.replace("width:200px;", "min-width:200px;");
    fs::write(CSS, css)?;

    Ok(())
}

pub fn run_cmd<A>(cmdline: A) -> IO<()>
where
    A: IntoIterator,
    A::Item: AsRef<OsStr>,
{
    let mut line = cmdline.into_iter();

    run(Command::new(line.next().expect("Command name")).args(line))
}

pub fn run(mut cmd: impl BorrowMut<Command>) -> IO<()> {
    let status = cmd.borrow_mut().status()?;
    if status.success() {
        Ok(())
    } else {
        Err(io::Error::new(
            io::ErrorKind::Other,
            format!("Subprocess failed: {status:?}"),
        ))
    }
}

pub fn doc_serve() -> IO<()> {
    use {
        axum::{
            http::StatusCode,
            response::{
                IntoResponse,
                Redirect,
            },
            routing::{
                get,
                get_service,
            },
            Router,
        },
        std::{
            net::SocketAddr,
            sync::{
                Arc,
                Mutex,
            },
        },
        tokio::{
            runtime::Runtime,
            sync::oneshot::{
                channel,
                Receiver,
                Sender,
            },
        },
        tower_http::services::ServeDir,
    };

    type Sn<T> = Arc<Mutex<T>>;
    fn sn<T>(t: T) -> Sn<T> {
        Arc::new(Mutex::new(t))
    }

    struct AppData {
        stop_tx: Option<Sender<()>>,
    }
    impl AppData {
        fn from(stop_tx: Sender<()>) -> Self {
            Self {
                stop_tx: Some(stop_tx),
            }
        }
    }

    type App = Sn<AppData>;
    fn app() -> (Receiver<()>, App) {
        let (stop_tx, stop_rx) = channel();
        (stop_rx, sn(AppData::from(stop_tx)))
    }

    async fn stopper(app: App) -> Result<(), String> {
        let mut lock = app.try_lock().map_err(|_| "Failed to lock state mutex")?;
        if let Some(tx) = lock.stop_tx.take() {
            tx.send(()).map_err(|_| "Failed to send stop signal")?;
        }
        Ok(())
    }

    fn my_app() -> (Receiver<()>, Router) {
        let (stop_rx, state) = app();

        let redir = get(|| async { Redirect::temporary("/bio/index.html") });
        let serve = get_service(ServeDir::new("target/doc")).handle_error(handle_error);
        let stop = tower::service_fn({
            let state = Arc::clone(&state);
            move |_| {
                let state = Arc::clone(&state);
                stopper(state)
            }
        });
        let stop = get_service(stop).handle_error(handle_error);

        let rout = Router::new()
            .route("/", redir)
            .route("/stop", stop)
            .with_state(state)
            .fallback_service(serve);

        (stop_rx, rout)
    }

    async fn handle_error<E: fmt::Debug>(err: E) -> impl IntoResponse {
        (StatusCode::INTERNAL_SERVER_ERROR, format!("Err: {err:?}"))
    }

    async fn serve(app: Router, stop_rx: Receiver<()>, port: u16) {
        let addr = SocketAddr::from(([127, 0, 0, 1], port));
        eprintln!("listening on {}", addr);
        let stop = async { stop_rx.await.unwrap() };
        axum::Server::bind(&addr)
            .serve(app.into_make_service())
            .with_graceful_shutdown(stop)
            .await
            .unwrap();
    }

    //https://docs.rs/hyper/latest/hyper/server/struct.Server.html#method.with_graceful_shutdown
    let (stop_rx, my_app) = my_app();
    Runtime::new()
        .unwrap()
        .block_on(serve(my_app, stop_rx, 19000));

    Ok(())
}
