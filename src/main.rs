use i3ipc::{reply::*, I3Connection};

use launchpad::mk2::*;
use launchpad::RGBColor;

use std::path::Path;

pub mod config;
use config::*;

mod error;
pub use error::*;

pub mod ui;
use ui::*;

pub mod i3;

fn main() -> crate::Result<()> {
    let config = config::Config::from_file(&Path::new("config.toml"))?;
    let mut program = Program::from_config(config)?;

    program.run()
}

pub struct Program {
    config: Config,
    connection: I3Connection,
    launchpad: MidiLaunchpadMk2,
}

impl Program {
    pub fn from_config(config: Config) -> crate::Result<Program> {
        Ok(Program {
            config,
            connection: I3Connection::connect()?,
            launchpad: MidiLaunchpadMk2::autodetect()?,
        })
    }

    pub fn run(&mut self) -> crate::Result<()> {
        let mut last_buffer = None;
        loop {
            for event in self.launchpad.poll() {
                self.handle_event(&event)?;
            }

            let mut buffer = LaunchpadBuffer::default();
            self.render(&mut buffer)?;

            if Some(&buffer) != last_buffer.as_ref() {
                buffer.render(&mut self.launchpad)?;
                last_buffer = Some(buffer);
            }

            std::thread::sleep(std::time::Duration::from_millis(15));
        }
    }

    pub fn render(&mut self, buffer: &mut LaunchpadBuffer) -> crate::Result<()> {
        let root = self.connection.get_tree()?;
        let workspaces = i3::find_workspaces(&root);

        for (x, (_workspace, windows)) in workspaces.iter().take(8).enumerate() {
            for (y, window) in windows.iter().take(8).enumerate() {
                let (x, y) = ui::rotate_coords((x as u8, y as u8), &self.config.side);
                let class: &str =
                    &window.window_properties.as_ref().unwrap()[&WindowProperty::Class];

                let color = if let Some(color) = &self.config.class_colors.get(class) {
                    color
                } else {
                    &self.config.default_color
                };

                let color = RGBColor(color.0, color.1, color.2);

                buffer.set(&Location::Pad(x as u8, y as u8), &color);
            }
        }

        Ok(())
    }

    pub fn handle_event(&mut self, event: &Event) -> crate::Result<()> {
        let root = self.connection.get_tree()?;
        let workspaces = i3::find_workspaces(&root);

        match event {
            Event::Press(Location::Pad(x, y)) => {
                let (x, y) = ui::unrotate_coords((*x, *y), &self.config.side);
                if let Some((workspace, windows)) = workspaces.get(x as usize) {
                    if let Some(window) = &windows.get(y as usize) {
                        self.connection
                            .run_command(&format!("[con_id=\"{}\"] focus", window.id))?;
                    } else {
                        self.connection
                            .run_command(&format!("workspace {}", workspace))?;
                    }
                }
            }
            _ => {}
        }

        Ok(())
    }
}
