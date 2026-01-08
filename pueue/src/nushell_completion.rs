use std::io::Write;

use clap::Command;
use clap_complete::Generator;

/// A custom Nushell completion generator that properly handles numeric types.
///
/// The upstream `clap_complete_nushell` library doesn't correctly map Rust numeric types
/// to Nushell's `int` type. Instead, all positional arguments without a specific `ValueHint`
/// are mapped to `string`. This custom generator wraps the standard generator and
/// post-processes the output to fix these type mappings.
pub struct PueueNushell;

impl Generator for PueueNushell {
    fn file_name(&self, name: &str) -> String {
        format!("{name}.nu")
    }

    fn generate(&self, cmd: &Command, buf: &mut dyn Write) {
        self.try_generate(cmd, buf)
            .expect("failed to write completion file");
    }

    fn try_generate(&self, cmd: &Command, buf: &mut dyn Write) -> Result<(), std::io::Error> {
        // First, generate the standard nushell completions
        let mut temp_buf = Vec::new();
        clap_complete_nushell::Nushell.try_generate(cmd, &mut temp_buf)?;

        // Convert to string for post-processing
        let mut completions = String::from_utf8(temp_buf)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;

        // Fix type mappings for known numeric arguments
        // The pattern we're looking for is: "task_id: string" or "task_id_1: string" etc.
        // These should be "task_id: int" instead.
        completions = fix_numeric_types(completions);

        buf.write_all(completions.as_bytes())
    }
}

/// Post-process the generated completions to fix numeric type mappings.
///
/// This function replaces `: string` with `: int` for arguments that are known to be
/// numeric types (usize, i32, etc.) based on their names.
fn fix_numeric_types(completions: String) -> String {
    let mut result = completions;

    // List of argument names that should be int type (not string)
    // These correspond to usize or other numeric types in the CLI definition
    let numeric_args = [
        "task_id",
        "task_id_1",
        "task_id_2",
        "task_ids",
        "parallel_tasks",
    ];

    for arg_name in numeric_args {
        // Match patterns like:
        // "    task_id: string" -> "    task_id: int"
        // "    ...task_ids: string" -> "    ...task_ids: int"
        // With optional "?" for optional arguments

        // Pattern 1: Regular argument
        let pattern = format!("{arg_name}: string");
        let replacement = format!("{arg_name}: int");
        result = result.replace(&pattern, &replacement);

        // Pattern 2: Rest argument (...)
        let pattern_rest = format!("...{arg_name}: string");
        let replacement_rest = format!("...{arg_name}: int");
        result = result.replace(&pattern_rest, &replacement_rest);

        // Pattern 3: Optional argument (?)
        let pattern_opt = format!("{arg_name}?: string");
        let replacement_opt = format!("{arg_name}?: int");
        result = result.replace(&pattern_opt, &replacement_opt);
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fix_numeric_types() {
        let input = r#"export extern "pueue switch" [
    --help(-h)                # Print help
    task_id_1: string         # The first task id
    task_id_2: string         # The second task id
  ]"#;

        let expected = r#"export extern "pueue switch" [
    --help(-h)                # Print help
    task_id_1: int         # The first task id
    task_id_2: int         # The second task id
  ]"#;

        assert_eq!(fix_numeric_types(input.to_string()), expected);
    }

    #[test]
    fn test_fix_numeric_types_rest_args() {
        let input = "    ...task_ids: string       # The task ids to be removed";
        let expected = "    ...task_ids: int       # The task ids to be removed";

        assert_eq!(fix_numeric_types(input.to_string()), expected);
    }

    #[test]
    fn test_fix_numeric_types_optional() {
        let input = "    task_id?: string       # Optional task id";
        let expected = "    task_id?: int       # Optional task id";

        assert_eq!(fix_numeric_types(input.to_string()), expected);
    }

    #[test]
    fn test_fix_numeric_types_parallel_tasks() {
        let input = "    parallel_tasks: string    # The amount of parallel tasks";
        let expected = "    parallel_tasks: int    # The amount of parallel tasks";

        assert_eq!(fix_numeric_types(input.to_string()), expected);
    }
}
