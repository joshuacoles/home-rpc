use serde_json::Value;

use std::process::Command;
use std::process::Stdio;

use std::io::Write;
use std::path::PathBuf;

pub struct ExecContext(pub PathBuf);

#[derive(Serialize, Deserialize)]
pub struct ExecData {
    command: String,
    data: Value,
}

#[derive(Serialize, Deserialize)]
pub struct ScheduleData {
    when: String,
    data: ExecData,
}

pub enum ExecError {
    InvalidCommand,
    NoSuchCommand,
    ExecutionError(std::io::Error),
    ScheduleError(std::io::Error),
}

impl ExecError {
    pub fn description(&self) -> String {
        match self {
            ExecError::InvalidCommand => "Invalid command name",
            ExecError::NoSuchCommand => "No such command",
            ExecError::ExecutionError(_) | ExecError::ScheduleError(_) => "An unknown error occurred"
        }.into()
    }
}

fn get_script(ExecContext(prefix): &ExecContext, command: &String) -> Result<PathBuf, ExecError> {
    if !command.chars().all(char::is_alphanumeric) {
        return Err(ExecError::InvalidCommand);
    }

    let mut pb = prefix.clone();
    pb.push(command);

    if !pb.exists() {
        return Err(ExecError::NoSuchCommand);
    }

    Ok(pb)
}

pub fn execute(context: &ExecContext, ExecData { command, data }: ExecData) -> Result<(), ExecError> {
    let command = get_script(context, &command)?;
    let data = serde_json::to_string(&data).unwrap();

    let output = Command::new(command)
        .arg(data)
        .output();

    match output {
        Ok(_) => Ok(()),
        Err(err) => Err(ExecError::ExecutionError(err))
    }
}

pub fn schedule(context: &ExecContext, ScheduleData { when, data: ExecData { command, data } }: ScheduleData)  -> Result<(), ExecError> {
    let command = get_script(context, &command)?;
    let command: &str = command.to_str().unwrap();
    let data = serde_json::to_string(&data).unwrap();

    let child = Command::new("/usr/bin/at")
        .stdin(Stdio::piped())
        .arg(when)
        .spawn();

    let mut child = match child {
        Ok(child) => child,
        Err(err) => return Err(ExecError::ScheduleError(err)),
    };

    // Command
    write!(child.stdin.as_mut().unwrap(),
           "{command} '{data}'",
           command = command,
           data = serde_json::to_string(&data).unwrap()
    ).expect("Error passing command to `at`");

    let output = child.wait_with_output();

    match output {
        Ok(_) => Ok(()),
        Err(err) => Err(ExecError::ExecutionError(err))
    }
}
