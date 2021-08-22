extern crate alsa;
extern crate libc;

use crate::res::Res;
use std::ffi::CString;
use alsa::seq;

pub struct Seq {
    seq: seq::Seq,
}

impl Seq {
    pub fn create(client_name: &str) -> Res<Self> {
        let seq = seq::Seq::open(None, None, true)?;
        seq.set_client_name(&CString::new(client_name)?)?;

        Ok(Seq { seq })
    }

    pub fn create_port(&self, name: &str, port_type: seq::PortType, capabilities: Option<seq::PortCap>) -> Res<seq::PortInfo> {
        let mut port_info = seq::PortInfo::empty()?;
        port_info.set_name(&CString::new(name)?);
        port_info.set_type(port_type);
        if let Some(c) = capabilities { port_info.set_capability(c) }
        self.seq.create_port(&port_info)?;
        Ok(port_info)
    }

    pub fn port_info(&self, addr: seq::Addr) -> Res<seq::PortInfo> {
        Ok(self.seq.get_any_port_info(addr)?)
    }

    pub fn clients(&self) -> impl Iterator<Item=seq::ClientInfo> + '_ {
        seq::ClientIter::new(&self.seq)
    }

    pub fn client_ports(&self, client_info: &seq::ClientInfo) -> impl Iterator<Item=seq::PortInfo> + '_ {
        seq::PortIter::new(&self.seq, client_info.get_client())
    }

    pub fn ports(&self) -> impl Iterator<Item=seq::PortInfo> + '_ {
        self.clients().flat_map(move |client_info| self.client_ports(&client_info))
    }

    pub fn subscriptions(&self, addr: seq::Addr, query_subs_type: seq::QuerySubsType) -> impl Iterator<Item=seq::PortSubscribe> + '_ {
        seq::PortSubscribeIter::new(&self.seq, addr, query_subs_type)
    }

    pub fn connect(&self, sender: seq::Addr, dest: seq::Addr) -> Res<(bool, seq::PortSubscribe)> {
        for ps in self.subscriptions(sender, seq::QuerySubsType::READ) {
            if ps.get_sender() == sender && ps.get_dest() == dest {
                return Ok((false, ps));
            }
        }

        let port_subscribe = seq::PortSubscribe::empty()?;
        port_subscribe.set_sender(sender);
        port_subscribe.set_dest(dest);
        self.seq.subscribe_port(&port_subscribe)?;

        Ok((true, port_subscribe))
    }

    pub fn play_note(
        &self,
        port_subscribe: &seq::PortSubscribe,
        note: u8,
    ) -> Res<u32> {
        let mut event = seq::Event::new(seq::EventType::Noteon, &seq::EvNote {
            channel: 0,
            duration: 100,
            note,
            off_velocity: 0,
            velocity: 70,
        });

        event.set_dest(port_subscribe.get_dest());
        event.set_direct();
        event.set_priority(true);
        // event.set_source(port_subscribe.get_sender().port);

        Ok(self.seq.event_output_direct(&mut event)?)
    }

    pub fn watch_events<'a>(
        &self,
        init: Option<Box<dyn Fn() -> Res<()> + 'a>>,
        handle_event: Box<dyn Fn(&seq::Event) -> Res<()> + 'a>,
    ) -> Res<()> {
        let mut input = self.seq.input();

        use alsa::PollDescriptors;
        let mut fds = Vec::<libc::pollfd>::new();
        let seqp = (&self.seq, Some(alsa::Direction::Capture));
        fds.append(&mut seqp.get()?);

        init.map(|c| c());

        loop {
            alsa::poll::poll(&mut fds, 1000)?;
            while input.event_input_pending(true)? != 0 {
                handle_event(&input.event_input()?).expect("handling event failed");
            }
        };
    }
}
