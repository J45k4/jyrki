use std::fs;

use serde_json::Value;

struct Property {
	name: String,
	description: String,
	mandatory: bool,
}

#[derive(Debug)]
struct Parameter {
	name: String,
	description: String,
	typ: String,
	mandatory: bool,
}


#[derive(Debug)]
struct ToolDef {
	name: String,
	description: String,
	parameters: Vec<Parameter>,
}

impl ToolDef {
	fn get_parameters(&self) -> Value {
		serde_json::json!(
			{
				"type": "object",
				"required": self.parameters.iter().filter(|param| param.mandatory).map(|param| Value::String(param.name.to_string())).collect::<Vec<Value>>(),
				"properties": self.parameters.iter().map(|param| {
					(param.name.to_string(), serde_json::json!(
						{
							"type": param.typ.to_string(),
							"description": param.description.to_string()
						}
					))
				}).collect::<serde_json::Map<String, Value>>()
			}
		)
	}
}

fn snake_to_pascal_case(s: &str) -> String {
    s.split('_')
        .map(|word| {
            let mut c = word.chars();
            match c.next() {
                Some(first_char) => first_char.to_uppercase().collect::<String>() + c.as_str(),
                None => String::new(),
            }
        })
        .collect()
}

fn snake_to_human_case(s: &str) -> String {
	s.split('_')
		.map(|word| {
			let mut c = word.chars();
			match c.next() {
				Some(first_char) => first_char.to_uppercase().collect::<String>() + c.as_str(),
				None => String::new(),
			}
		})
		.collect::<Vec<String>>().join(" ")
}

pub fn main() {
	let tools_data = fs::read_to_string("tools.json").unwrap();
	let tools: serde_json::Value = serde_json::from_str(&tools_data).unwrap();
	// println!("Tools: {:#?}", tools);

	if !tools.is_array() {
		panic!("tools.json must be an array of tool definitions");
	}
	
	let mut tool_defs = Vec::<ToolDef>::new();

	for tool in tools.as_array().unwrap() {
		let typ = tool.get("type").unwrap().as_str().unwrap();

		match typ {
			"function" => {
				let function = tool.get("function").unwrap();
				let name = function.get("name").unwrap().as_str().unwrap();
				let description = function.get("description").unwrap().as_str().unwrap();
				println!("Function: {} - {}", name, description);

				let mut tool = ToolDef {
					name: name.to_string(),
					description: description.to_string(),
					parameters: Vec::<Parameter>::new(),
				};

				let parameters = function.get("parameters").unwrap().as_object().unwrap();
				let required = parameters.get("required").unwrap().as_array().unwrap();
				let properties = parameters.get("properties").unwrap().as_object().unwrap();

				for (prop_name, prop_data) in properties {
					let description = prop_data.get("description").unwrap().as_str().unwrap();
					let mandatory = required.contains(&Value::String(prop_name.to_string()));

					let parameter = Parameter {
						name: prop_name.to_string(),
						typ: prop_data.get("type").unwrap().as_str().unwrap().to_string(), 
						description: description.to_string(),
						mandatory: mandatory,
					};

					tool.parameters.push(parameter);
				}

				tool_defs.push(tool);
			},
			_ => {
				panic!("Unknown tool type: {}", typ);
			}
		}
	}

	// println!("Tool definitions: {:#?}", tool_defs);

	let tool_enum = format!("#[derive(Debug, Clone, PartialEq)]\npub enum Tool {{\n{}\n}}", tool_defs.iter().map(|tool| {
		format!("\t{}", snake_to_pascal_case(&tool.name))
	}).collect::<Vec<String>>().join(",\n"));

	let tool_to_string_impl = format!(r#"impl ToString for Tool {{
	fn to_string(&self) -> String {{
		match self {{
{}
		}}
	}}
}}"#, tool_defs.iter().map(|tool| {
		let name = snake_to_pascal_case(&tool.name);
		let human_str = snake_to_human_case(&tool.name);
		format!("\t\t\tTool::{} => \"{}\".to_string(),", name, human_str)
	}).collect::<Vec<String>>().join("\n"));

	let tool_impls = format!(r#"impl Tool {{
	pub fn get_name(&self) -> &str {{
		match self {{
{}
		}}
	}}

	pub fn get_description(&self) -> &str {{
		match self {{
{}
		}}
	}}

	pub fn get_parameters(&self) -> serde_json::Value {{
		match self {{
{}
		}}
	}}
}}"#, tool_defs.iter().map(|tool| {
		let name = snake_to_pascal_case(&tool.name);
		format!("\t\t\tTool::{} => \"{}\",", name, tool.name)
	}).collect::<Vec<String>>().join("\n"),
	tool_defs.iter().map(|tool| {
		let name = snake_to_pascal_case(&tool.name);
		format!("\t\t\tTool::{} => \"{}\",", name, tool.description)
	}).collect::<Vec<String>>().join("\n"),
	tool_defs.iter().map(|tool| {
		let name = snake_to_pascal_case(&tool.name);
		format!("\t\t\tTool::{} => serde_json::json!({}),", name, tool.get_parameters())
	}).collect::<Vec<String>>().join("\n"));


	let tools_array = format!("pub const TOOLS: [Tool; {}] = [\n{}\n];", tool_defs.len(), tool_defs.iter().map(|tool| {
		let name = snake_to_pascal_case(&tool.name);
		format!("\tTool::{},", name)
	}).collect::<Vec<String>>().join("\n"));

	let tools_string = [tool_enum, tool_to_string_impl, tool_impls, tools_array].join("\n\n");

	let structs = tool_defs.iter().map(|tool| {
		let name = tool.name.to_string();
		let pascal_name = snake_to_pascal_case(&name);
		let parameters = tool.parameters.iter().map(|param| {
			let param_name = param.name.to_string();
			let rust_type = match param.typ.as_str() {
				"string" => "String",
				"number" => "f32",
				"integer" => "u32",
				"boolean" => "bool",
				_ => "Value",
			};
			let param_type = if param.mandatory {
				format!("{}", rust_type)
			} else {
				format!("Option<{}>", rust_type)
			};
			format!("\tpub {}: {},", param_name, param_type)
		}).collect::<Vec<String>>().join("\n");

		format!(r#"#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct {} {{
{}
}}"#, pascal_name, parameters)
	}).collect::<Vec<String>>().join("\n\n");

	let tool_call_impls = format!(r#"impl ToolCallParameters {{
	pub fn get_name(&self) -> &str {{
		match self {{
{}
		}}
	}}

	pub fn get_args(&self) -> String {{
		match self {{
{}
		}}
	}}

	pub fn parse(name: &str, args: &str) -> anyhow::Result<ToolCallParameters> {{
		match name {{
{}
			_ => anyhow::bail!("Unknown tool: {{}}", name),
		}}
	}}
}}"#, tool_defs.iter().map(|tool| {
		format!("\t\t\tToolCallParameters::{}(_) => \"{}\",", snake_to_pascal_case(&tool.name), tool.name)
	}).collect::<Vec<String>>().join("\n"),
	tool_defs.iter().map(|tool| {
		let name = snake_to_pascal_case(&tool.name);
		format!("\t\t\tToolCallParameters::{}(args) => serde_json::to_string(args).unwrap(),", name)
	}).collect::<Vec<String>>().join("\n"),
	tool_defs.iter().map(|tool| {
		let name = snake_to_pascal_case(&tool.name);
		format!("\t\t\t\"{}\" => Ok(ToolCallParameters::{}(serde_json::from_str(args)?)),", tool.name, name)
	}).collect::<Vec<String>>().join("\n"));

	let code = format!(r#"
{}
{}
#[derive(Debug, Clone)]
pub enum ToolCallParameters {{
	{}
}}

{}"#, tools_string, structs, tool_defs.iter().map(|tool| {
		let name = tool.name.to_string();
		let pascal_name = snake_to_pascal_case(&name);
		format!("{}({})", pascal_name, pascal_name)
	}).collect::<Vec<String>>().join(",\n\t"), tool_call_impls);

	println!("{}", code);

	fs::write("src/generated.rs", code).unwrap();
}