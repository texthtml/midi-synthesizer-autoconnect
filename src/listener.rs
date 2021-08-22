use crate::seq::Seq;
use crate::res::Res;
use alsa::seq;

pub struct Listener<'a, T: ListenerHandler> where {
    pub seq: &'a Seq,
    handler: T,
}

pub trait ListenerHandler {
    fn initialize(&self) -> Res<()>;

    fn handle(&self, event: &seq::Event) -> Res<()>;
}

impl<'a, T: ListenerHandler> Listener<'a, T> {
    pub fn create(handler: T, seq: &'a Seq) -> Res<Self> {
        Ok(Listener {
            seq,
            handler,
        })
    }

    pub fn start(&self) -> Res<()> {
        let listener_port_info = self.seq.create_port(
            "Listener",
            seq::PortType::APPLICATION,
            Some(seq::PortCap::WRITE),
        )?;

        self.seq.connect(seq::Addr::system_announce(), listener_port_info.addr())?;

        self.seq.watch_events(
            Some(Box::new(|| self.handler.initialize())),
            Box::new(|ev| self.handler.handle(ev)),
        )
    }
}
