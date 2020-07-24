extern crate gtk;
extern crate glib;
extern crate gio;
extern crate dirs;
extern crate regex;

use gtk::prelude::*;
use glib::clone;
use gio::prelude::*;
use gtk::{ApplicationWindow, AboutDialog, Builder, Button, Switch, Entry, FileChooserButton, ComboBoxText};
use regex::Regex;
use std::env::args;
use std::rc::Rc;
use std::cell::RefCell;
use std::fs::File;
use std::io::prelude::*;
use notify_rust::Notification;

#[derive(Debug)]
#[derive(Clone)]
struct DesktopData {
	app_name: String,
	comment: String,
	exec_command: String,
	icon_filename: String,
	categories: String,
	extensions: String,
	app_type: String,
	terminal: bool
}

impl DesktopData {
	fn new () -> DesktopData {
		return DesktopData {
			app_name: "".to_string(),
			comment: "".to_string(),
			exec_command: "".to_string(),
			icon_filename: "".to_string(),
			categories: "".to_string(),
			extensions: "".to_string(),
			app_type: "".to_string(),
			terminal: false
		};
	}

	fn validate(&self) -> Result<(), &str> {
		if self.app_name.trim().len() == 0 {
			return Err("Missing app name");
		} else if self.exec_command.trim().len() == 0 {
			return Err("Missing exec command");
		} else if self.icon_filename.trim().len() == 0 {
			return Err("Missing icon");
		}
		
		return Ok(());
	}

	fn save_file(&self) {
		match dirs::data_local_dir() {
			Some(mut path) => {
				let re = Regex::new(r"\s+").unwrap();
				let name: String = re.replace_all(&self.app_name.trim().to_lowercase(), "-").to_string();
				
				path.push("applications");
				path.push(name);
				path.set_extension("desktop");

				let file_path_str = path.clone().into_os_string().into_string().unwrap();

				let mut file = File::create(path).unwrap();
				let mut file_data: String = String::from("[Desktop Entry]");
				file_data.push_str("\nType=");
				file_data.push_str(self.app_type.as_str());
				file_data.push_str("\nName=");
				file_data.push_str(self.app_name.trim());
		
				if self.comment.trim().len() > 0 {
					file_data.push_str("\nComment=");
					file_data.push_str(self.comment.trim());
				}
		
				if self.icon_filename.trim().len() > 0 {
					file_data.push_str("\nIcon=");
					file_data.push_str(self.icon_filename.trim());
				}
		
				if self.categories.trim().len() > 0 {
					file_data.push_str("\nCategories=");
					file_data.push_str(self.categories.trim());
				}
		
				if self.extensions.trim().len() > 0 {
					file_data.push_str("\nMimeType=");
					file_data.push_str(self.extensions.trim());
				}
		
				file_data.push_str("\nExec=");
				file_data.push_str(self.exec_command.trim());
				file_data.push_str("\nTerminal=");
				file_data.push_str(self.terminal.to_string().as_str());
				file.write_all(file_data.as_str().as_bytes()).unwrap();
		
				Notification::new()
					.summary("Desktop file created")
					.body(format!("The file has been created at {}", file_path_str).as_str())
					.icon("document-new")
					.show().unwrap();
			}
			None => {
				Notification::new()
					.summary("Error")
					.body("Could not find the local directory for .desktop files")
					.icon("document-new")
					.show().unwrap();	
			}
		}
	
		
	}
}

fn build_ui(application: &gtk::Application) {
    let glade_src = include_str!("res/app_ui.glade");
    let builder = Builder::from_string(glade_src);

    let window: ApplicationWindow = builder.get_object("main_window").expect("Couldn't get main_window");
    window.set_application(Some(application));

	let app_name_entry: Entry = builder.get_object("app_name_text").expect("app_name_text not found");
	let app_comment_entry: Entry = builder.get_object("comment_text").expect("comment_text not found");
	let app_exec_entry: Entry = builder.get_object("exec_command_text").expect("exec_command_text not found");
	let app_categories_entry: Entry = builder.get_object("categories_text").expect("categories_text not found");
	let app_extensions_entry: Entry = builder.get_object("extensions_text").expect("extensions_text not found");
	let type_combo: ComboBoxText= builder.get_object("type_combo").expect("type_combo not found");
	let app_icon_file: FileChooserButton = builder.get_object("app_icon_file_chooser").expect("app_icon_file_chooser not found");
	let terminal_switch: Switch = builder.get_object("terminal_toggle").expect("terminal_toggle not found");
	let create_button: Button = builder.get_object("create_button").expect("Couldn't get create_button");

	let open_about_dialog: Button = builder.get_object("open_about_dialog_button").expect("open_about_dialog_button not found");
	let about_dialog: AboutDialog = builder.get_object("about_dialog").expect("about_dialog not found");

	about_dialog.connect_delete_event(|about_dialog, _| {
		about_dialog.hide();
		gtk::Inhibit(true)
	});

	open_about_dialog.connect_clicked(clone!(@weak about_dialog => move |_| about_dialog.show_all()));
	

	// Disables the button
	//create_button.set_sensitive(false);

	let desktop_data: Rc<RefCell<DesktopData>> = Rc::new(RefCell::new(DesktopData::new()));

	create_button.connect_clicked(clone!(
	@weak app_name_entry, 
	@weak app_comment_entry => move |_|{

		let mut dd = desktop_data.borrow_mut();
		dd.app_name = app_name_entry.get_text().to_string();
		dd.app_type = type_combo.get_active_text().unwrap().to_string();
		dd.terminal = terminal_switch.get_active();
		dd.comment = app_comment_entry.get_text().to_string();
		dd.exec_command = app_exec_entry.get_text().to_string();
		dd.categories = app_categories_entry.get_text().to_string();
		dd.extensions = app_extensions_entry.get_text().to_string();

		if let Some(filename) = app_icon_file.get_filename() {
			let file_str = filename.into_os_string().into_string().unwrap();
			dd.icon_filename = file_str;
		}

		println!("{:?}", dd);

		match dd.validate() {
			Ok(_) => dd.save_file(),
			Err(error) => println!("Error: {}", error)
		}

	}));

    window.show_all();
}

fn main () {
	let application = gtk::Application::new (
		Some("com.github.darkbox.rocket"),
		Default::default()	
	)
	.expect("Initialization failed...");

	application.connect_activate(|app|{
		build_ui(app);	
	});

	application.run(&args().collect::<Vec<_>>());
}
