use crate::res::Res;
use crate::seq::Seq;
use crate::listener;

pub struct Synchronizer {
    seq: Seq
}

impl listener::ListenerHandler for &Synchronizer {
    fn initialize(&self) -> Res<()> {
        tracing::info!("Synchronizer started");

        tracing::info!("{}", match self.connect_all()? {
            (true, true) => "Waiting for more MIDI sources",
            (true, false) => "Waiting for MIDI sources",
            (false, true) => "Waiting for a synthetizer",
            (false, false) => "Waiting for a synthetizer and MIDI sources",
        });

        Ok(())
    }

    fn handle(&self, ev: &alsa::seq::Event<'_>) -> Res<()> {
        match ev.get_type() {
            alsa::seq::EventType::PortStart => {
                Ok(self.auto_connect(ev.get_data().expect("asd"))?)
            },
            _ => {
                tracing::trace!("event {:?}", ev);
                Ok(())
            }
        }
    }
}

impl Synchronizer {
    pub fn create() -> Res<Self> {
        Ok(Synchronizer { seq: Seq::create("Synchronizer")? })
    }

    pub fn start(&self) -> Res<()> {
        listener::Listener::create(self, &self.seq)?.start()
    }

    fn auto_connect(&self, addr: alsa::seq::Addr) -> Res<()> {
        let port_info = self.seq.port_info(addr)?;

        if Synchronizer::is_synthetizer(&port_info) {
            self.detected("Synthetizer", &port_info)?;

            let mut no_midi_sources = true;

            for midi_source in self.midi_sources() {
                no_midi_sources = false;
                self.connect(midi_source.addr(), port_info.addr()).expect("connection failed");
            }

            if no_midi_sources {
                tracing::warn!("No MIDI sources available");
            }
        } else if Synchronizer::is_midi_source(&port_info) {
            self.detected("Midi source", &port_info)?;

            if let Some(synthetizer) = self.synthetizer() {
                self.connect(port_info.addr(), synthetizer.addr()).expect("connection failed");
            } else {
                tracing::warn!("Cannot connect: no synthetizer available");
            }
        } else {
            self.detected("Unknown seq port", &port_info)?;
        }

        Ok(())
    }

    fn connect_all(&self) -> Res<(bool, bool)> {
        let midi_sources : Vec<_> = self.midi_sources().collect();
        let no_midi_sources = midi_sources.is_empty();

        if let Some(synthetizer) = self.synthetizer() {
            self.detected("Synthetizer", &synthetizer)?;

            for midi_source in midi_sources {
                self.detected("Midi source", &midi_source)?;
                self.connect(midi_source.addr(), synthetizer.addr()).expect("connection failed");
            }

            if no_midi_sources {
                tracing::warn!("No MIDI sources available");
            }

            Ok((true, !no_midi_sources))
        } else {
            tracing::warn!("Cannot connect: no synthetizer available");

            for midi_source in midi_sources {
                self.detected("Midi source", &midi_source)?;
            }

            if no_midi_sources {
                tracing::warn!("No MIDI sources available");
            }

            Ok((false, !no_midi_sources))
        }
    }

    fn connect(&self, sender: alsa::seq::Addr, synthetizer: alsa::seq::Addr) -> Res<(bool, alsa::seq::PortSubscribe)> {
        self.seq.connect(sender, synthetizer).and_then(|(new, port_subscribe)| {
            if new {
                tracing::info!(
                    "{} has been connected to {}",
                    self.seq.port_info(port_subscribe.get_sender())?.get_name()?,
                    self.seq.port_info(port_subscribe.get_dest())?.get_name()?,
                );

                self.seq.play_note(&port_subscribe, 60)?;
                self.seq.play_note(&port_subscribe, 64)?;
                self.seq.play_note(&port_subscribe, 69)?;
            }

            Ok((new, port_subscribe))
        })
    }

    fn synthetizer(&self) -> Option<alsa::seq::PortInfo> {
        self.seq.ports().find(Synchronizer::is_synthetizer)
    }

    fn midi_sources(&self) -> impl Iterator<Item=alsa::seq::PortInfo> + '_ {
        self.seq.ports().filter(Synchronizer::is_midi_source)
    }

    fn is_synthetizer(port_info: &alsa::seq::PortInfo) -> bool {
        port_info.get_type().contains(alsa::seq::PortType::SYNTHESIZER)
    }

    fn is_midi_source(port_info: &alsa::seq::PortInfo) -> bool {
        port_info.get_type().contains(alsa::seq::PortType::MIDI_GENERIC) &&
        port_info.get_capability().contains(alsa::seq::PortCap::READ)
    }

    fn detected(&self, type_name: &str, port_info: &alsa::seq::PortInfo) -> Res<()> {
        tracing::info!(
            "{} detected: {}",
            type_name,
            self.seq.port_info(port_info.addr())?.get_name()?,
        );
        tracing::debug!(
            "{} types: {:?}",
            self.seq.port_info(port_info.addr())?.get_name()?,
            port_info.get_type(),
        );
        tracing::debug!(
            "{} capabilities: {:?}",
            self.seq.port_info(port_info.addr())?.get_name()?,
            port_info.get_capability(),
        );

        Ok(())
    }
}
