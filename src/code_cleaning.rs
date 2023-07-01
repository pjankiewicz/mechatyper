pub fn apply_indentation(old_code: &str, new_code: &str) -> String {
    let old_code_lines: Vec<&str> = old_code.lines().collect();
    let new_code_lines: Vec<&str> = new_code.lines().collect();

    // Detect the indentation in old code
    let old_indentation = old_code_lines
        .iter()
        .filter(|line| !line.trim().is_empty())
        .map(|line| {
            line.chars()
                .take_while(|c| c.is_whitespace())
                .collect::<String>()
        })
        .next()
        .unwrap_or_default();

    // Detect the number of leading whitespace characters in new code
    let new_indentation_count = new_code_lines
        .iter()
        .filter(|line| !line.trim().is_empty())
        .map(|line| line.chars().take_while(|c| c.is_whitespace()).count())
        .min()
        .unwrap_or(0);

    // Apply indentation to new code
    let indented_code = new_code_lines
        .into_iter()
        .map(|line| {
            if line.trim().is_empty() {
                line.to_string()
            } else {
                old_indentation.clone() + &line[new_indentation_count..]
            }
        })
        .collect::<Vec<String>>()
        .join("\n");

    // Ensure the output ends with a newline character
    if indented_code.ends_with('\n') {
        indented_code
    } else {
        indented_code + "\n"
    }
}

pub fn extract_python_code(input: &str) -> Option<String> {
    let mut lines = input.lines();
    let mut python_code = String::new();
    let mut in_python_code_block = false;

    while let Some(line) = lines.next() {
        if line.trim_start().starts_with("```python") {
            in_python_code_block = true;
        } else if line.trim_start().starts_with("```") {
            if in_python_code_block {
                // Reached the end of the Python code block
                break;
            }
        } else if in_python_code_block {
            python_code.push_str(line);
            python_code.push('\n');
        }
    }

    if !python_code.is_empty() {
        Some(python_code)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_apply_indentation_with_spaces() {
        let old_code = "    def old_function():\n        print('This is the old function')\n";
        let new_code = "def new_function():\n    print('This is the new function')\n";

        let expected_indented_new_code =
            "    def new_function():\n        print('This is the new function')\n";
        let indented_new_code = apply_indentation(old_code, new_code);

        assert_eq!(indented_new_code, expected_indented_new_code);
    }

    #[test]
    fn test_apply_indentation_with_tabs() {
        let old_code = "\tdef old_function():\n\t\tprint('This is the old function')\n";
        let new_code = "def new_function():\n\tprint('This is the new function')\n";

        let expected_indented_new_code =
            "\tdef new_function():\n\t\tprint('This is the new function')\n";
        let indented_new_code = apply_indentation(old_code, new_code);

        assert_eq!(indented_new_code, expected_indented_new_code);
    }

    #[test]
    fn test_extract_python_code() {
        let code = extract_python_code(
            r#"
Sure! Here's a Python function that takes a list and returns the sum of its elements:

```python
def calculate_sum(lst):
return sum(lst)
```

You can use this function by passing your list as an argument to the function `calculate_sum()`. The `sum()` function takes an iterable (like a list) as input and returns the sum of its elements."#,
        );
        println!("{:?}", code);
    }
}
