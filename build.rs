use std::{fs::{self, DirEntry}, path::Path};

pub fn main() -> anyhow::Result<()> {
    let compiler = shaderc::Compiler::new().unwrap();
    let options = shaderc::CompileOptions::new().unwrap();
    let shader_files = recurse_dir("./assets/shader")?;

    for file in shader_files {
        let path = file.path();
        let source = fs::read_to_string(path.clone())?;
        let file_name = path.to_string_lossy().to_string();
        let extension = file_name.split(".").last().expect("shader files must have an extension");
        let shader_binary = compiler.compile_into_spirv(
            &source,
            extension_to_shader_kind(extension),
            &file_name,
            "main",
            Some(&options),
        )?;
        fs::write(Path::new(format!("{}_{extension}.spv", path.with_extension("").to_string_lossy().to_string()).as_str()), shader_binary.as_binary_u8())?;
    }

    Ok(())
}

fn extension_to_shader_kind(extension: &str) -> shaderc::ShaderKind {
    match extension {
        "frag" => shaderc::ShaderKind::Fragment,
        "vert" => shaderc::ShaderKind::Vertex,
        _ => panic!("unsupported shader kind"),
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
