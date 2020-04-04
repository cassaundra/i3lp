pub mod config;
use config::*;

use i3ipc::{reply::*, I3Connection};

use launchpad::mk2::*;
use launchpad::*;

use std::collections::HashMap;

fn main() {
    let mut connection = I3Connection::connect().unwrap();
    let mut launchpad = MidiLaunchpadMk2::autodetect().unwrap();

    println!("{}", connection.get_version().unwrap().human_readable);

    // config
    let mut colors: HashMap<String, RGBColor> = HashMap::new();
    colors.insert("Firefox".to_string(), RGBColor::new(0xFF, 0x6A, 0x11));
    colors.insert("Emacs".to_string(), RGBColor::new(0xC1, 0x33, 0xFF));
    colors.insert("Thunderbird".to_string(), RGBColor::new(0x1D, 0x2D, 0xB1));
    colors.insert("Spotify".to_string(), RGBColor::new(0x28, 0xFF, 0x73));
    colors.insert("discord".to_string(), RGBColor::new(0x51, 0x71, 0xFF));
    colors.insert("Xfce4-terminal".to_string(), RGBColor::new(0x16, 0x17, 0x20));

    let mut program = Program::from_config(Config {
        colors,
    });

    // let mut last_workspace_windows = None;
    let mut last_buffer = None;

    loop {
        let workspaces = connection.get_workspaces().unwrap().workspaces;

        let root: i3ipc::reply::Node = connection.get_tree().unwrap();
        let workspace_windows = find_workspace_windows(&root);

        for event in launchpad.poll() {
            program.handle_event(&event, &mut connection, &workspaces, &workspace_windows);
        }

        let mut buffer = LaunchpadBuffer::default();
        program.render(&mut buffer, &workspaces, workspace_windows);

        if Some(&buffer) != last_buffer.as_ref() {
            buffer.render(&mut launchpad).unwrap();
            last_buffer = Some(buffer);
        }

        std::thread::sleep(std::time::Duration::from_millis(5));
    }
}

#[derive(Clone, Default, PartialEq)]
pub struct LaunchpadBuffer {
    grid: [[RGBColor; 8]; 8],
    top_buttons: [RGBColor; 8],
    right_buttons: [RGBColor; 8],
}

impl LaunchpadBuffer {
    pub fn set(&mut self, location: &Location, color: &RGBColor) {
        use Location::*;

        match location {
            Pad(x, y) => self.grid[*y as usize][*x as usize] = *color,
            Button(c, ButtonSide::Top) => self.top_buttons[*c as usize] = *color,
            Button(c, ButtonSide::Right) => self.right_buttons[*c as usize] = *color,
        }
    }

    pub fn render(&self, launchpad: &mut impl LaunchpadMk2) -> launchpad::Result<()> {
        let mut buffer = Vec::with_capacity(80);

        for y in 0..8 {
            for x in 0..8 {
                buffer.push((Location::Pad(x as u8, y as u8), self.grid[y][x]));
            }
        }

        for i in 0..8 {
            buffer.push((Location::Button(i as u8, ButtonSide::Top), self.top_buttons[i]));
            buffer.push((Location::Button(i as u8, ButtonSide::Right), self.right_buttons[i]));
        }

        launchpad.light_multi_rgb(buffer)
    }
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
                    let class: &str =
                        &window.window_properties.as_ref().unwrap()[&WindowProperty::Class];

                    let color = if let Some(color) = &self.config.colors.get(class) {
                        *color.clone()
                    } else {
                        RGBColor::new(0x80, 0x80, 0x80)
                    };

                    let color = RGBColor(color.0, color.1, color.2);

                    launchpad.set(&Location::Pad(x as u8, y as u8), &color);
                }
            }
        }
    }

    pub fn handle_event(&mut self, event: &Event, connection: &mut I3Connection, workspaces: &Vec<Workspace>, workspace_windows: &HashMap<String, Vec<&Node>>) {
        match event {
            Event::Press(Location::Pad(x, y)) => {
                if let Some(workspace) = workspaces.get(*x as usize) {
                    if let Some(windows) = &workspace_windows.get(&workspace.name) {
                        if let Some(window) = windows.get(*y as usize) {
                            connection
                                .run_command(&format!("[con_id=\"{}\"] focus", window.id))
                                .unwrap();
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
    }
}

pub fn find_workspace_windows(node: &Node) -> HashMap<String, Vec<&Node>> {
    let mut layout: HashMap<String, Vec<&Node>> = HashMap::new();
    for (workspace, window) in find_workspace_windows_rec(node, None) {
        layout.entry(workspace).or_default().push(window)
    }
    layout
}

fn find_workspace_windows_rec(node: &Node, mut workspace: Option<String>) -> Vec<(String, &Node)> {
    let mut windows = vec![];

    if is_workspace(node) {
        workspace = node.name.clone();
    } else if is_window(node) && workspace.is_some() {
        windows.push((workspace.clone().unwrap(), node));
    }

    for child in &node.nodes {
        windows.extend(find_workspace_windows_rec(child, workspace.clone()));
    }

    windows
}

fn is_workspace(node: &Node) -> bool {
    node.nodetype == NodeType::Workspace
}

fn is_window(node: &Node) -> bool {
    if let Some(properties) = &node.window_properties {
        if properties.get(&WindowProperty::Class) != Some(&String::from("i3bar"))
            && node.nodetype == NodeType::Con
            && node.nodes.is_empty()
        {
            return true;
        }
    }

    false
}
