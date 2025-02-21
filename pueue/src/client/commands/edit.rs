use std::{
    collections::BTreeMap,
    env,
    fs::{File, create_dir, read_to_string},
    io::Write,
    path::{Path, PathBuf},
};

use pueue_lib::{client::Client, error::Error, network::message::*, settings::Settings};
use tempfile::tempdir;

use super::handle_response;
use crate::{
    client::style::OutputStyle, internal_prelude::*, process_helper::compile_shell_command,
};

/// This function handles the logic for editing tasks.
/// At first, we request the daemon to send us the task to edit.
/// This also results in the task being `Locked` on the daemon side, preventing it from being
/// started or manipulated in any way, as long as we're editing.
///
/// After receiving the task information, the user can then edit it in their editor.
/// Upon exiting the text editor, the line will then be read and sent to the server
pub async fn edit(client: &mut Client, style: &OutputStyle, task_ids: Vec<usize>) -> Result<()> {
    // Request the data to edit from the server and issue a task-lock while doing so.
    let init_message = Request::EditRequest(task_ids);
    client.send_request(init_message).await?;

    let init_response = client.receive_response().await?;

    // In case we don't receive an EditResponse, something went wrong.
    // Handle the response and return.
    let Response::Edit(editable_tasks) = init_response else {
        handle_response(style, init_response)?;
        return Ok(());
    };

    let task_ids: Vec<usize> = editable_tasks.iter().map(|task| task.id).collect();
    let result = edit_tasks(&client.settings, editable_tasks);

    // Any error while editing will result in the client aborting the editing process.
    // However, as the daemon moves tasks that're edited into the `Locked` state, we cannot simply
    // exit the client. We rather have to notify the daemon that the editing process was
    // interrupted. In the following, we notify the daemon of any errors, so it can restore the
    // tasks to their previous state.
    let editable_tasks = match result {
        Ok(editable_tasks) => editable_tasks,
        Err(error) => {
            eprintln!("Encountered an error while editing. Trying to restore the task's status.");
            // Notify the daemon that something went wrong.
            let edit_message = Request::EditRestore(task_ids);
            client.send_request(edit_message).await?;

            let response = client.receive_response().await?;
            match response {
                Response::Failure(message) | Response::Success(message) => {
                    eprintln!("{message}");
                }
                _ => eprintln!("Received unknown response: {response:?}"),
            };

            return Err(error);
        }
    };

    // Send the edited tasks back to the daemon.
    client
        .send_request(Request::EditedTasks(editable_tasks))
        .await?;

    let response = client.receive_response().await?;
    handle_response(style, response)?;

    Ok(())
}

/// This is a small generic wrapper around the editing logic.
///
/// There're two different editing modes in Pueue, one file based and on toml based.
/// Call the respective function based on the editing mode.
pub fn edit_tasks(
    settings: &Settings,
    editable_tasks: Vec<EditableTask>,
) -> Result<Vec<EditableTask>> {
    // Create the temporary directory that'll be used for all edits.
    let temp_dir = tempdir().context("Failed to create temporary directory for edtiting.")?;
    let temp_dir_path = temp_dir.path();

    match settings.client.edit_mode {
        pueue_lib::settings::EditMode::Toml => {
            edit_tasks_with_toml(settings, editable_tasks, temp_dir_path)
        }
        pueue_lib::settings::EditMode::Files => {
            edit_tasks_with_folder(settings, editable_tasks, temp_dir_path)
        }
    }
}

/// This editing mode creates a temporary folder that contains a single `tasks.toml` file.
///
/// This file contains all tasks to be edited with their respective properties.
/// While this is very convenient, users must make sure to not malform the content and respect toml
/// based escaping as not doing so could lead to deserialization errors or broken/misbehaving
/// task commands.
pub fn edit_tasks_with_toml(
    settings: &Settings,
    editable_tasks: Vec<EditableTask>,
    temp_dir_path: &Path,
) -> Result<Vec<EditableTask>> {
    // Convert to map for nicer representation and serialize to toml.
    // The keys of the map must be strings for toml to work.
    let map: BTreeMap<String, EditableTask> = BTreeMap::from_iter(
        editable_tasks
            .into_iter()
            .map(|task| (task.id.to_string(), task)),
    );
    let toml = toml::to_string(&map)
        .map_err(|err| Error::Generic(format!("\nFailed to serialize tasks to toml:\n{err}")))?;
    let temp_file_path = temp_dir_path.join("tasks.toml");

    // Write the file to disk and open it with the editor.
    std::fs::write(&temp_file_path, toml).map_err(|err| {
        Error::IoPathError(temp_file_path.clone(), "creating temporary file", err)
    })?;
    run_editor(settings, &temp_file_path)?;

    // Read the data back from disk into the map and deserialize it back into a map.
    let content = read_to_string(&temp_file_path)
        .map_err(|err| Error::IoPathError(temp_file_path.clone(), "reading temporary file", err))?;
    let map: BTreeMap<String, EditableTask> = toml::from_str(&content)
        .map_err(|err| Error::Generic(format!("\nFailed to deserialize tasks to toml:\n{err}")))?;

    Ok(map.into_values().collect())
}

/// This editing mode creates a temporary folder in which one subfolder is created for each task
/// that should be edited.
/// Those task folders then contain a single file for each of the task's editable properties.
/// This approach allows one to edit properties without having to worry about potential file
/// formats or other shennanigans.
pub fn edit_tasks_with_folder(
    settings: &Settings,
    mut editable_tasks: Vec<EditableTask>,
    temp_dir_path: &Path,
) -> Result<Vec<EditableTask>> {
    for task in editable_tasks.iter() {
        task.create_temp_dir(temp_dir_path)?
    }

    run_editor(settings, temp_dir_path)?;

    // Read the data back from disk into the struct.
    for task in editable_tasks.iter_mut() {
        task.read_temp_dir(temp_dir_path)?
    }

    Ok(editable_tasks)
}

/// Open the folder that contains all files for editing in the user's `$EDITOR`.
/// Returns as soon as the editor is closed again.
/// Get the editor that should be used from the environment.
pub fn run_editor(settings: &Settings, temp_dir: &Path) -> Result<()> {
    let editor = match env::var("EDITOR") {
        Err(_) => bail!("The '$EDITOR' environment variable couldn't be read. Aborting."),
        Ok(editor) => editor,
    };

    // Compile the command to start the editor on the temporary file.
    // We escape the file path for good measure, but it shouldn't be necessary.
    let path = shell_escape::escape(temp_dir.to_string_lossy());
    let editor_command = format!("{editor} {path}");
    let mut modified_settings = settings.clone();
    modified_settings.daemon.env_vars.insert(
        "PUEUE_EDIT_PATH".to_string(),
        temp_dir.to_string_lossy().to_string(),
    );
    let status = compile_shell_command(&modified_settings, &editor_command)
        .status()
        .context("Editor command did somehow fail. Aborting.")?;

    if !status.success() {
        bail!("The editor exited with a non-zero code. Aborting.");
    }

    Ok(())
}

/// Implements convenience functions to serialize and deserialize editable tasks to and from disk
/// so users can edit the task via their editor.
trait Editable {
    fn create_temp_dir(&self, temp_dir: &Path) -> Result<()>;
    fn read_temp_dir(&mut self, temp_dir: &Path) -> Result<()>;
}

impl Editable for EditableTask {
    /// Create a folder for this task that contains one file for each editable property.
    fn create_temp_dir(&self, temp_dir: &Path) -> Result<()> {
        let task_dir = temp_dir.join(self.id.to_string());
        create_dir(&task_dir)
            .map_err(|err| Error::IoPathError(task_dir.clone(), "creating task dir", err))?;

        // Create command file
        let cmd_path = task_dir.join("command");
        let mut output = File::create(&cmd_path)
            .map_err(|err| Error::IoPathError(cmd_path.clone(), "creating command file", err))?;
        write!(output, "{}", self.command)
            .map_err(|err| Error::IoPathError(cmd_path.clone(), "writing command file", err))?;

        // Create cwd file
        let cwd_path = task_dir.join("path");
        let mut output = File::create(&cwd_path).map_err(|err| {
            Error::IoPathError(cwd_path.clone(), "creating temporary path file", err)
        })?;
        write!(output, "{}", self.path.to_string_lossy())
            .map_err(|err| Error::IoPathError(cwd_path.clone(), "writing path file", err))?;

        // Create label  file. If there's no label, create an empty file.
        let label_path = task_dir.join("label");
        let mut output = File::create(&label_path).map_err(|err| {
            Error::IoPathError(label_path.clone(), "creating temporary label file", err)
        })?;
        if let Some(label) = &self.label {
            write!(output, "{}", label)
                .map_err(|err| Error::IoPathError(label_path.clone(), "writing label file", err))?;
        }

        // Create priority file. If there's no priority, create an empty file.
        let priority_path = task_dir.join("priority");
        let mut output = File::create(&priority_path).map_err(|err| {
            Error::IoPathError(priority_path.clone(), "creating priority file", err)
        })?;
        write!(output, "{}", self.priority).map_err(|err| {
            Error::IoPathError(priority_path.clone(), "writing priority file", err)
        })?;

        Ok(())
    }

    /// Read the edited files of this task's temporary folder back into this struct.
    ///
    /// The user has finished editing at this point in time.
    fn read_temp_dir(&mut self, temp_dir: &Path) -> Result<()> {
        let task_dir = temp_dir.join(self.id.to_string());

        // Read command file
        let cmd_path = task_dir.join("command");
        let command = read_to_string(&cmd_path)
            .map_err(|err| Error::IoPathError(cmd_path.clone(), "reading command file", err))?;
        // Make sure the command isn't empty.
        if command.trim().is_empty() {
            bail!("Found empty command after edit for task {}", self.id);
        }
        self.command = command.trim().to_string();

        // Read cwd file
        let cwd_path = task_dir.join("path");
        let cwd = read_to_string(&cwd_path)
            .map_err(|err| Error::IoPathError(cwd_path.clone(), "reading path file", err))?;
        let cwd = cwd.trim();
        // Make sure the path isn't empty
        if cwd.trim().is_empty() {
            bail!("Found empty path after edit for task {}", self.id);
        }
        let path = PathBuf::from(&cwd);
        // Make sure the path actually exists
        if !self.path.exists() {
            bail!(
                "Found non-existing path '{:?}' after edit for task {}",
                self.path,
                self.id
            );
        }
        self.path = path;

        // Read label file. If file is empty, set the label to `None`
        let label_path = task_dir.join("label");
        let label = read_to_string(&label_path)
            .map_err(|err| Error::IoPathError(label_path.clone(), "reading label file", err))?;
        self.label = if label.trim().is_empty() {
            None
        } else {
            Some(label.trim().to_string())
        };

        // Read priority file. If file is empty, set the priority to `None`
        let priority_path = task_dir.join("priority");
        let priority = read_to_string(&priority_path).map_err(|err| {
            Error::IoPathError(priority_path.clone(), "reading priority file", err)
        })?;
        // Parse the user input into a usize.
        self.priority = priority.trim().parse().context(format!(
            "Failed to parse priority string '{}' into an integer for task {}",
            priority, self.id
        ))?;

        Ok(())
    }
}
