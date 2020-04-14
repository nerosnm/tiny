mod bridge;

use gio::prelude::*;
use gtk::prelude::*;

use bridge::ClientBridge;

pub fn main() {
    let application = gtk::Application::new(Some("com.github.osa1.tiny"), Default::default())
        .expect("Initialization failed...");

    application.connect_activate(|app| {
        // TODO: Is this ever signalled more than once?

        // bridge passed to callbacks
        let (bridge, rcv_client_ev) = ClientBridge::new();
        let window = build_ui(app, bridge);
        rcv_client_ev.attach(None, move |msg| {
            window.show_all();
            glib::Continue(true)
        });
    });

    application.run(&std::env::args().collect::<Vec<_>>());
}

fn build_ui(application: &gtk::Application, bridge: ClientBridge) -> gtk::Widget {
    let notebook = gtk::Notebook::new();
    notebook.set_tab_pos(gtk::PositionType::Bottom);

    let test = gtk::Label::new(Some("just testing"));

    notebook.append_page(&test, None::<&gtk::Widget>);

    let window = gtk::ApplicationWindow::new(application);

    window.set_title("tiny");
    window.set_decorated(false);
    window.set_default_size(200, 200);
    window.add(&notebook);
    window.show_all();

    window.upcast()
}
