use std::io::Write;
use std::process::Command;
use std::process::Stdio;

use itertools::Itertools;

pub fn execute(values: Vec<String>, expr: &str) -> Result<(Vec<String>, String), String> {
    let mut cmd = Command::new("sh")
        .arg("-c")
        .arg(expr)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| e.to_string())?;

    cmd.stdin
        .as_mut()
        .unwrap() // FIXME unwrap for mut option
        .write_all((values.join("\n") + "\n").as_bytes())
        .map_err(|e| e.to_string())?;

    let cmd_result = cmd.wait_with_output();
    let output = cmd_result.map_err(|e| e.to_string())?;
    let stderr: String = String::from_utf8_lossy(&output.stderr).into();
    let stdout: String = String::from_utf8_lossy(&output.stdout).into();

    Ok(
        (stdout.lines().filter(|x| !x.is_empty())
             .map(str::to_owned).collect_vec(),
         stderr)
    )
}

#[cfg(test)]
mod tests {
    use crate::shell_runner::execute;

    #[test]
    fn execute_sort_with_empty_list() {
        assert_eq!(
            execute(vec![], "sort"),
            Ok((vec![], "".to_string()))
        )
    }

    #[test]
    fn execute_sort_with_2_1() {
        assert_eq!(
            execute(vec!["2".to_string(), "1".to_string()], "sort"),
            Ok((vec!["1".to_string(), "2".to_string()], "".to_string()))
        )
    }

    #[test]
    fn execute_unknown_command() {
        assert_eq!(
            execute(vec![], "unk-cmd"),
            Ok((vec![], "sh: unk-cmd: command not found\n".to_string()))
        )
    }
}