use i3ipc::reply::*;

use std::collections::BTreeMap;

pub fn find_workspaces(node: &Node) -> Vec<(String, Vec<&Node>)> {
    let mut layout: BTreeMap<String, Vec<&Node>> = BTreeMap::new();
    for (workspace, window) in find_workspaces_rec(node, None) {
        layout.entry(workspace).or_default().push(window)
    }
    layout.into_iter().collect()
}

fn find_workspaces_rec(node: &Node, mut workspace: Option<String>) -> Vec<(String, &Node)> {
    let mut windows = vec![]; // TODO with capacity?

    if is_workspace(node) {
        workspace = node.name.clone();
    } else if is_window(node) && workspace.is_some() {
        windows.push((workspace.clone().unwrap(), node));
    }

    for child in &node.nodes {
        windows.extend(find_workspaces_rec(child, workspace.clone()));
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
