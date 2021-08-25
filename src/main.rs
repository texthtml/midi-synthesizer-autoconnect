mod res;
mod seq;
mod listener;
mod synchronizer;

fn start() -> res::Res<()> {
    synchronizer::Synchronizer::create()?.start()
}

fn main() -> res::Res<()> {
    if std::env::var(tracing_subscriber::EnvFilter::DEFAULT_ENV).is_err() {
        std::env::set_var(tracing_subscriber::EnvFilter::DEFAULT_ENV, "midi_synthesizer_autoconnect=info");
    }

    tracing_subscriber::fmt::init();

    let res = start();

    if let Err(e) = &res {
        tracing::error!("{}", e);
    }

    res
}
