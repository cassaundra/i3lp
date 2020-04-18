use i3ipc::{reply::*, I3Connection};

use std::collections::HashMap;

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
