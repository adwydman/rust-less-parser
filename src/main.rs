use std::fs::File;
use std::io::Read;
use std::collections::HashMap;

const IMPORT: &str = "@import";
const VARIABLE_DECLARATOR: &str = "@";
const SELECTOR_OPEN: &str = "{";
const SELECTOR_CLOSE: &str = "}";

struct SelectorNode<'a> {
    name: String,
    properties: &'a HashMap<String, String>,
    // children: Vec<SelectorNode>,
    parent: &'a Option<Box<SelectorNode<'a>>>
}

#[derive(Debug)]
enum ASTNode {
    Variable(String, String),    // name, value
    Rule(String, HashMap<String, String>),  // selector, properties
}

fn extract_variables_from_line(line: &str) -> Result<(String, String), ()> { 
    let parts: Vec<&str> = line.split(':').collect();
    if parts.len() == 2 {
        let variable_name = parts[0].trim().to_string();
        let variable_value = parts[1].trim().trim_end_matches(';').to_string();

        return Ok((variable_name, variable_value));
    }

    Err(())
}

fn extract_properties_from_selector<'a>(
    lines: &'a Vec<&'a str>,
    mut i: usize, 
    mut lines_inside_selector: usize, 
    parent_selector: &Option<Box<SelectorNode>>
    // mut selector_openers: i32,
    // mut selector_closers: i32
) -> (String, HashMap<String, String>, &'a Option<Box<SelectorNode<'a>>>) {
    let line = lines[i];
    let selector = line.trim_end_matches(SELECTOR_OPEN).trim().to_string();
    let mut properties: HashMap<String, String> = HashMap::new();
    let empty_properties: HashMap<String, String> = HashMap::new();

    let mut selector_node = Some(
        Box::new(SelectorNode { 
            name: selector.to_string(),
            parent: parent_selector,
            properties: &empty_properties
        })
    );

    match parent_selector {
        Some(parent) => {
            println!("parent name: {}", parent.name);
        },
        None => {
            println!("none");
        }
    }

    loop {
        let next_line = lines[i + 1 + lines_inside_selector];

        if next_line.trim() == SELECTOR_CLOSE {
            i += lines_inside_selector;
            break;
        }

        if next_line.ends_with(SELECTOR_OPEN) {
            lines_inside_selector += 1;
            extract_properties_from_selector(lines, i + 2, lines_inside_selector, &selector_node);
        } else {
            let property_value_pair: Vec<&str> = next_line.split(':').collect();
            let property = property_value_pair[0].trim().to_string();
            let value = property_value_pair[1].replace(";", "").trim().to_string();
    
            properties.insert(property, value);
    
            lines_inside_selector += 1;
        }
    }

    if let Some(selector_node) = &mut selector_node {
        selector_node.properties = &properties;
    } else {
        // Handle the case where selector_node is None
    }

    (selector, properties.clone(), &selector_node)
}

fn parse_less(less_code: &str) -> Vec<ASTNode> {
    let mut ast = Vec::new();
    let lines: Vec<&str> = less_code.lines().collect();

    let mut i = 0;
    
    while i < lines.len() {
        let trimmed_line = lines[i].trim();

        if trimmed_line.is_empty() {
            i += 1;
            continue;
        }

        if trimmed_line.starts_with(IMPORT) { // Handle @import, duh!
            let split_import: Vec<&str> = trimmed_line.split(" ").collect();
            let imported_file_path = split_import[1].replace("'", "").replace(";", "");

            match get_file_content(imported_file_path.as_str()) {
                Ok(less_code) => {
                    let imported_file_ast = parse_less(less_code.as_str());

                    for node in imported_file_ast {
                        ast.push(node);
                    }
                }
                Err(()) => {
                    println!("An error occurred");
                }
            }    
        } else if trimmed_line.starts_with(VARIABLE_DECLARATOR) { // Handle variable declarations
            match extract_variables_from_line(trimmed_line) {
                Ok((variable_name, variable_value)) => {
                    ast.push(ASTNode::Variable(variable_name, variable_value));
                    i += 1;
                    continue;
                },
                Err(()) => () // can't extract
            }
        }

        if trimmed_line.ends_with(SELECTOR_OPEN) { // assumes the selector and `{` are on the same line

            let mut selector_openers = 0;
            let mut selector_closers = 0;
            let mut lines_inside_selector = 0;
            
            let (
                selector,
                properties,
                selector_node
            ) = extract_properties_from_selector(
                &lines,
                i,
                lines_inside_selector,
                &None
                // selector_openers,
                // selector_closers
            );

            ast.push(ASTNode::Rule(selector, properties));
        }
        i += 1;
    }

    ast
}

fn generate_css(ast: Vec<ASTNode>) -> String {
    let mut css = String::new();
    let mut variables: HashMap<String, String> = HashMap::new();

    for node in ast {
        match node {
            ASTNode::Variable(name, value) => {
                variables.insert(name, value);
            },
            ASTNode::Rule(selector, properties) => {
                css.push_str(&format!("{} {{\n", selector));
                for (property_name, property_value) in properties {
                    let mut property_value_vector: Vec<&str> = property_value.split(' ').collect();

                    for split_value in property_value_vector.iter_mut() {
                        if split_value.starts_with(VARIABLE_DECLARATOR) {
                            let split_value_string = split_value.to_string();

                            match variables.get(&split_value_string) {
                                Some(value) => *split_value = value,
                                None => (), // todo: key not found
                            }
                        }
                    }

                    let parsed_property_value = property_value_vector.join(" ");

                    css.push_str(&format!("  {}: {};\n", property_name, parsed_property_value));
                }
                css.push_str("}\n");
            },
        }
    }
    css
}

fn get_file_content(filename: &str) -> Result<String, ()> {
    let mut full_filepath = String::from("./src/");
    full_filepath.push_str(filename);

    let mut file = match File::open(full_filepath) {
        Ok(file) => file,
        Err(error) => {
            println!("Failed to open file: {}", error);
            return Err(());
        }
    };
    
    let mut content = String::new();
    if let Err(error) = file.read_to_string(&mut content) {
        println!("Failed to read file: {}", error);
        return Err(());
    }

    Ok(content)
}

fn main() {
    match get_file_content("input.less") {
        Ok(less_code) => {
            let ast = parse_less(less_code.as_str());
            let css = generate_css(ast);

            println!("Generated CSS:\n{}", css);
        }
        Err(()) => {
            println!("An error occurred");
        }
    }    
}
