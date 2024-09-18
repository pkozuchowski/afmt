#[cfg(test)]
mod tests {
    use afmt::config::*;
    use similar::{ChangeTag, TextDiff};
    use std::path::Path;
    use std::process::Command;

    #[test]
    fn source_target_tests() {
        for entry in std::fs::read_dir("tests/prettier").unwrap() {
            let entry = entry.unwrap();
            let source_path = entry.path();
            if source_path.extension().and_then(|ext| ext.to_str()) == Some("cls") {
                println!("### Processing file: {:?}", source_path);
                process_test_file(&source_path);
            }
        }
    }

    fn process_test_file(source_path: &Path) {
        let file_path = source_path
            .to_str()
            .expect("PathBuf to String failed.")
            .to_string();
        let session = Session::new(Config::default(), vec![file_path.clone()]);
        let vec = session.format();
        let output = vec
            .into_iter()
            .next()
            .and_then(|result| result.ok())
            .expect("format result failed.");

        let prettier_output = run_prettier(source_path).expect("Failed to run Prettier");

        if output != prettier_output {
            let file_content =
                std::fs::read_to_string(&file_path).expect("Failed to read the file content.");

            println!("\nFailed: {:?}:", file_path);
            println!("-------------------------------------\n");
            println!("{}", file_content);
            println!("-------------------------------------\n");
            print_side_by_side_diff(&output, &prettier_output, source_path);
            println!("\n-------------------------------------\n");

            assert_eq!(
                output,
                prettier_output,
                "Mismatch in {}",
                source_path.display()
            );
        }
    }

    fn run_prettier(source_path: &Path) -> Result<String, String> {
        let output = Command::new("npx")
            .arg("prettier")
            .arg("--plugin=prettier-plugin-apex")
            .arg("--parser=apex")
            .arg(source_path.to_str().unwrap())
            .output()
            .expect("Failed to execute Prettier");

        if output.status.success() {
            let formatted_code =
                String::from_utf8(output.stdout).expect("Prettier output is not valid UTF-8");
            Ok(formatted_code)
        } else {
            let error_message = String::from_utf8(output.stderr)
                .unwrap_or_else(|_| "Unknown error while running Prettier".to_string());
            Err(error_message)
        }
    }

    fn print_side_by_side_diff(output: &str, expected: &str, source_path: &Path) {
        let diff = TextDiff::from_lines(expected, output);
        let mut left_col = String::new();
        let mut right_col = String::new();

        // Header for the side-by-side view
        println!("{:<40} | {:<40}", "Prettier:", "Afmt:\n");

        for change in diff.iter_all_changes() {
            match change.tag() {
                ChangeTag::Delete => {
                    left_col = format!("\x1b[91m- {:<38}\x1b[0m", change.to_string().trim_end()); // Red for deletions (left)
                    right_col = String::from(""); // Empty on the right side
                }
                ChangeTag::Insert => {
                    left_col = String::from(""); // Empty on the left side
                    right_col = format!("\x1b[92m+ {:<38}\x1b[0m", change.to_string().trim_end());
                    // Green for insertions (right)
                }
                ChangeTag::Equal => {
                    left_col = format!("  {:<38}", change.to_string().trim_end()); // No color for unchanged lines
                    right_col = format!("  {:<38}", change.to_string().trim_end());
                }
            }

            // Print the two columns side-by-side
            println!("{:<40} | {:<40}", left_col, right_col);
        }
    }
}
