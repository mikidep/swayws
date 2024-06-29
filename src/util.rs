use env_logger::filter;
use snafu::prelude::*;
use swayipc::Connection;
use swayipc::Workspace;

use crate::error::SwayWsError;
use crate::error::*;

pub fn focus_workspace(
    connection: &mut Connection,
    workspace_name: &str,
) -> Result<(), SwayWsError> {
    let command_text = format!("workspace {}", workspace_name);
    send_ipc_command(connection, &command_text)
}

pub fn move_workspace_to_output(
    connection: &mut Connection,
    workspace_name: &str,
    output_name: &str,
) -> Result<(), SwayWsError> {
    let command_text = format!(
        "workspace {0} output {1},\
        workspace {0},\
        move workspace to {1}",
        workspace_name, output_name
    );
    send_ipc_command(connection, &command_text)
}

pub fn rename_workspace(
    connection: &mut Connection,
    from: &str,
    to: &str,
) -> Result<(), SwayWsError> {
    let command_text = format!("rename workspace {from} to {to}");
    send_ipc_command(connection, &command_text)
}

pub fn send_ipc_command(
    connection: &mut Connection,
    command_text: &str,
) -> Result<(), SwayWsError> {
    let outcomes: Result<(), swayipc::Error> = connection
        .run_command(command_text)
        .context(SwayIpcCtx)?
        .into_iter()
        .collect();
    outcomes.context(SwayIpcCtx)
}

pub fn print_outputs(connection: &mut Connection) -> Result<(), SwayWsError> {
    let outputs = connection.get_outputs().context(SwayIpcCtx)?;
    println!("Outputs (name):");
    for monitor in outputs.into_iter() {
        println!("{}", monitor.name);
    }
    Ok(())
}

pub fn print_workspaces(connection: &mut Connection) -> Result<(), SwayWsError> {
    let workspaces: Vec<Workspace> = connection.get_workspaces().context(SwayIpcCtx)?;
    println!("Workspaces (id, name):");
    let fill = match workspaces.last() {
        Some(ws) => ws.num.to_string().len(),
        None => 1,
    };
    for ws in workspaces.into_iter() {
        println!("{0:>width$} {1:>width$}", ws.num, ws.name, width = fill);
    }
    Ok(())
}

pub fn get_workspace_current(connection: &mut Connection) -> Result<Workspace, SwayWsError> {
    let workspaces: Vec<Workspace> = connection.get_workspaces().context(SwayIpcCtx)?;
    Ok(workspaces
        .into_iter()
        .find(|w| w.focused)
        .expect("No focused workspace??"))
}

pub fn get_workspace_prev(connection: &mut Connection) -> Result<Workspace, SwayWsError> {
    let current_out = get_workspace_current(connection)?.output;
    let out_workspaces: Vec<Workspace> = connection
        .get_workspaces()
        .context(SwayIpcCtx)?
        .into_iter()
        .filter(|w| w.output == current_out)
        .collect();
    let mut out_ws_prev = out_workspaces.clone();
    let last = out_ws_prev.pop().expect("No workspace in output??");
    out_ws_prev.insert(0, last);
    let mut zipped = out_workspaces.iter().zip(out_ws_prev);
    Ok(zipped
        .find(|(w, _)| w.focused)
        .expect("No focused workspace??")
        .1)
}

pub fn get_workspace_next(connection: &mut Connection) -> Result<Workspace, SwayWsError> {
    let current_out = get_workspace_current(connection)?.output;
    let out_workspaces: Vec<Workspace> = connection
        .get_workspaces()
        .context(SwayIpcCtx)?
        .into_iter()
        .filter(|w| w.output == current_out)
        .collect();
    let mut out_ws_next = out_workspaces.clone();
    let first = out_ws_next.remove(0);
    out_ws_next.push(first);
    let mut zipped = out_workspaces.iter().zip(out_ws_next);
    Ok(zipped
        .find(|(w, _)| w.focused)
        .expect("No focused workspace??")
        .1)
}

pub fn get_workspace_special(
    name: String,
    connection: &mut Connection,
) -> Result<String, SwayWsError> {
    match name.as_str() {
        "current" => Ok(get_workspace_current(connection)?.name),
        "prev" => Ok(get_workspace_prev(connection)?.name),
        "next" => Ok(get_workspace_next(connection)?.name),
        _ => Ok(name),
    }
}

pub fn get_second_output(
    connection: &mut Connection,
    output_names: &[String],
) -> Result<swayipc::Output, SwayWsError> {
    let outputs = connection.get_outputs().context(SwayIpcCtx)?;
    if outputs.len() == 1 {
        return NoOutputMatchedCtx.fail();
    }
    outputs
        .into_iter()
        .find(|monitor| is_not_in_list(&monitor.name, output_names))
        .ok_or(NoOutputMatchedCtx.build())
}

pub fn is_not_in_list<V: Eq>(v: &V, list: &[V]) -> bool {
    for value in list.iter() {
        if *value == *v {
            return false;
        }
    }
    true
}
