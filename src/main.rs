use i3ipc::{I3Connection, reply::*};

use launchpad::mk2::*;
use launchpad::RGBColor;

use std::collections::HashMap;
use std::path::Path;

pub mod config;
use config::*;

mod error;
pub use error::*;

pub mod ui;
use ui::*;

pub mod i3;
use i3::*;

fn main() -> crate::Result<()> {
    let mut connection = I3Connection::connect()?;
    let mut launchpad = MidiLaunchpadMk2::autodetect()?;

    println!("{}", connection.get_version()?.human_readable);

    let config = config::Config::from_file(&Path::new("config.toml"))?;
    let mut program = Program::from_config(config);

    // let mut last_workspace_windows = None;
    let mut last_buffer = None;

    // TODO program flow
    loop {
        let workspaces = connection.get_workspaces()?.workspaces;

        let root: i3ipc::reply::Node = connection.get_tree()?;
        let workspace_windows = find_workspace_windows(&root);

        for event in launchpad.poll() {
            program.handle_event(&event, &mut connection, &workspaces, &workspace_windows)?;
        }

        let mut buffer = LaunchpadBuffer::default();
        program.render(&mut buffer, &workspaces, workspace_windows);

        if Some(&buffer) != last_buffer.as_ref() {
            buffer.render(&mut launchpad)?;
            last_buffer = Some(buffer);
        }

        std::thread::sleep(std::time::Duration::from_millis(5));
    }

    Ok(())
}

pub struct Program {
    config: Config,
}

impl Program {
    pub fn from_config(config: Config) -> Self {
        Program {
            config,
        }
    }

    pub fn render(
        &self, launchpad: &mut LaunchpadBuffer, workspaces: &Vec<Workspace>,
        workspace_windows: HashMap<String, Vec<&Node>>,
    ) {
        for (x, workspace) in workspaces.iter().take(8).enumerate() {
            if let Some(windows) = &workspace_windows.get(&workspace.name) {
                for (y, window) in windows.iter().take(8).enumerate() {
                    let class: &str = &window.window_properties.as_ref().unwrap()[&WindowProperty::Class];

                    let color = if let Some(color) = &self.config.class_colors.get(class) {
                        color
                    } else {
                        &self.config.default_color
                    };

                    let color = RGBColor(color.0, color.1, color.2);

                    launchpad.set(&Location::Pad(x as u8, y as u8), &color);
                }
            }
        }
    }

    pub fn handle_event(
        &mut self, event: &Event, connection: &mut I3Connection, workspaces: &Vec<Workspace>,
        workspace_windows: &HashMap<String, Vec<&Node>>,
    ) -> crate::Result<()> {
        match event {
            Event::Press(Location::Pad(x, y)) => {
                if let Some(workspace) = workspaces.get(*x as usize) {
                    if let Some(windows) = &workspace_windows.get(&workspace.name) {
                        if let Some(window) = windows.get(*y as usize) {
                            connection.run_command(&format!("[con_id=\"{}\"] focus", window.id))?;
                        } else {
                            connection
                                .run_command(&format!("workspace {}", workspace.name))
                                .unwrap();
                        }
                    }
                }
            }
            _ => {}
        }

        Ok(())
    }
}
