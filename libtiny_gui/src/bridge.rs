use libtiny_client::{Client, ServerInfo};
use libtiny_ui::MsgTarget;

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use std::thread::{spawn, JoinHandle};

/// `libtiny_ui::MsgTarget`, but owns the strings, to be able implement `Send`.
pub(crate) enum MsgTargetOwned {
    Server { serv: String },
    Chan { serv: String, chan: String },
    User { serv: String, nick: String },
    AllServTabs { serv: String },
    CurrentTab,
}

impl<'a> From<MsgTarget<'a>> for MsgTargetOwned {
    fn from(msg_target: MsgTarget) -> MsgTargetOwned {
        match msg_target {
            MsgTarget::Server { serv } => MsgTargetOwned::Server {
                serv: serv.to_owned(),
            },
            MsgTarget::Chan { serv, chan } => MsgTargetOwned::Chan {
                serv: serv.to_owned(),
                chan: chan.to_owned(),
            },
            MsgTarget::User { serv, nick } => MsgTargetOwned::User {
                serv: serv.to_owned(),
                nick: nick.to_owned(),
            },
            MsgTarget::AllServTabs { serv } => MsgTargetOwned::AllServTabs {
                serv: serv.to_owned(),
            },
            MsgTarget::CurrentTab => MsgTargetOwned::CurrentTab,
        }
    }
}

enum GUIMsg {
    Connect(ServerInfo),
    Reconnect {
        serv: String,
    },
    RawMsg {
        serv: String,
        msg: String,
    },
    Privmsg {
        msg_target: MsgTargetOwned,
        msg: String,
    },
    Join {
        serv: String,
        chan: String,
    },
    Part {
        serv: String,
        chan: String,
    },
    Nick {
        serv: String,
        new_nick: String,
    },
    Quit {
        serv: String,
    },
}

#[derive(Clone)]
pub(crate) struct ClientBridge(Rc<RefCell<ClientBridgeInner>>);

impl ClientBridge {
    pub(crate) fn new() -> (ClientBridge, glib::Receiver<libtiny_client::Event>) {
        let (bridge_inner, rcv_client_msg) = ClientBridgeInner::new();
        (
            ClientBridge(Rc::new(RefCell::new(bridge_inner))),
            rcv_client_msg,
        )
    }
}

struct ClientBridgeInner {
    snd_gui_msg: tokio::sync::mpsc::Sender<GUIMsg>,
    tokio_thread: JoinHandle<()>,
}

impl ClientBridgeInner {
    fn new() -> (ClientBridgeInner, glib::Receiver<libtiny_client::Event>) {
        let (snd_gui_msg, rcv_gui_msg) = tokio::sync::mpsc::channel(10);
        let (snd_client_msg, rcv_client_msg) =
            glib::MainContext::channel::<libtiny_client::Event>(glib::PRIORITY_DEFAULT);

        // Spawn tokio runtime
        let tokio_thread = spawn(|| {
            let mut runtime = tokio::runtime::Builder::new()
                .basic_scheduler()
                .enable_all()
                .build()
                .unwrap();
            let local = tokio::task::LocalSet::new();
            local.block_on(&mut runtime, bridge_task(rcv_gui_msg, snd_client_msg));
        });

        (
            ClientBridgeInner {
                snd_gui_msg,
                tokio_thread,
            },
            rcv_client_msg,
        )
    }
}

async fn bridge_task(
    mut rcv_gui_msg: tokio::sync::mpsc::Receiver<GUIMsg>,
    mut snd_client_msg: glib::Sender<libtiny_client::Event>,
) {
    let mut clients: HashMap<String, Client> = HashMap::new();
    while let Some(msg) = rcv_gui_msg.recv().await {
        match msg {
            GUIMsg::Connect(server_info) => todo!(),
            GUIMsg::Reconnect { serv } => todo!(),
            GUIMsg::RawMsg { serv, msg } => todo!(),
            GUIMsg::Privmsg { msg_target, msg } => todo!(),
            GUIMsg::Join { serv, chan } => todo!(),
            GUIMsg::Part { serv, chan } => todo!(),
            GUIMsg::Nick { serv, new_nick } => todo!(),
            GUIMsg::Quit { serv } => todo!(),
        }
    }
}
