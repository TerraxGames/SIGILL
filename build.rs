use std::{fs::{self, DirEntry}, path::Path};

#[allow(unused)]
macro_rules! p {
    ($($tokens: tt)*) => {
        println!("cargo:warning={}", format!($($tokens)*))
    }
}

pub fn main() -> anyhow::Result<()> {
    let compiler = shaderc::Compiler::new().unwrap();
    let mut options = shaderc::CompileOptions::new().unwrap();
    options.set_include_callback(|requested, include_type, source, include_depth| {
        if include_depth > 127 {
            return shaderc::IncludeCallbackResult::Err(format!("Maximum include depth reached in {source} including {requested}! Check for recursive include directives."))
        }
        if include_type == shaderc::IncludeType::Standard {
            return shaderc::IncludeCallbackResult::Err(format!("Cannot find requested {requested} from {source}!"))
        }
        let source = fs::read_to_string(format!("{source}/../{requested}")).expect(format!("Failed to find {requested} from {source}").as_str()).to_string();
        Ok(
            shaderc::ResolvedInclude {
                resolved_name: requested.to_string(),
                content: source,
            }
        )
    });
    let shader_files = recurse_dir("./assets/shader")?;

    for file in shader_files {
        let path = file.path();
        if let Some(file_name) = path.file_name() {
            if file_name.to_string_lossy().to_string().ends_with(".spv") {
                continue;
            }
        }
        let source = fs::read_to_string(path.clone())?;
        let file_name = path.to_string_lossy().to_string();
        let extension = file_name.split(".").last();
        if extension.is_none() {
            continue;
        }
        let shader_kind = extension_to_shader_kind(extension.unwrap());
        if shader_kind.is_none() {
            continue;
        }
        let shader_binary = compiler.compile_into_spirv(
            &source,
            shader_kind.unwrap(),
            &file_name,
            "main",
            Some(&options),
        )?;
        let target_path = &format!("{}_{}.spv", path.with_extension("").to_string_lossy().to_string(), extension.unwrap());
        fs::write(Path::new(target_path.as_str()), shader_binary.as_binary_u8())?;
    }

    Ok(())
}

fn extension_to_shader_kind(extension: &str) -> Option<shaderc::ShaderKind> {
    match extension {
        "frag" => Some(shaderc::ShaderKind::Fragment),
        "vert" => Some(shaderc::ShaderKind::Vertex),
        _ => None,
    }
}

fn recurse_dir(path: impl AsRef<Path>) -> std::io::Result<Vec<DirEntry>> {
    let mut entries = Vec::new();
    let dir = fs::read_dir(path)?;

    dir
        .into_iter()
        // Fallibly extract entry metadata
        .map(|entry| {
            let entry = entry?;
            Ok((entry.metadata()?, entry))
        })
        .collect::<std::io::Result<Vec<_>>>()?
        .into_iter()
        // Add files to entries and discard them
        .filter_map(|(metadata, entry)| {
            if metadata.is_file() {
                entries.push(entry);
                return None
            }
            metadata.is_dir().then_some(entry)
        })
        .collect::<Vec<_>>()
        .iter()
        // Recurse child directories
        .try_for_each(|entry| {
            entries.extend(recurse_dir(entry.path())?);
            Ok::<(), std::io::Error>(())
        })?;

    Ok(entries)
}
