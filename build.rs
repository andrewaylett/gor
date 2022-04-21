use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::exit;
use std::{env, fs, io};

fn main() {
    println!("cargo:rerun-if-changed=tests/compile/");

    let out: PathBuf = env::var("OUT_DIR")
        .expect("Cargo should set OUT_DIR")
        .into();
    let out = out.join("gen").join("compile");
    fs::create_dir_all(&out).expect("Can't create $OUT_DIR/gen/compile");

    let input = Path::new(".")
        .join("tests")
        .join("compile")
        .canonicalize()
        .expect("Cannot canonicalise");

    let inputs = input.read_dir().unwrap();

    let mut modules: Vec<String> = vec![];

    for file in inputs {
        match file {
            Ok(file) => {
                let name = file.file_name();
                if !name.to_string_lossy().ends_with(".go") {
                    println!("cargo:warning=Ignoring non-go file {:?}", name);
                    continue;
                }
                match name.into_string() {
                    Ok(name) => {
                        let include_file = input.join(&name);
                        generate_test_for_file(&name, &mut modules, &out, &include_file)
                            .expect("Failed to generate test");
                    }
                    Err(os_string) => {
                        println!(
                            "cargo:warning=Can't handle files with non-String names: {:?}",
                            os_string
                        );
                        exit(1);
                    }
                }
            }
            Err(e) => {
                println!("cargo:warning={:?}", e);
                exit(1);
            }
        }
    }

    let mut file = File::create(out.with_extension("rs")).expect("Failed to create module file");
    for module_name in modules {
        writeln!(file, "mod {};", module_name).expect("Failed to write line");
    }

    let mut base =
        File::create(out.with_file_name("base").with_extension("rs")).expect("Create failed");
    writeln!(base, "#[path = {:?}]", out.with_extension("rs")).expect("Write failed");
    writeln!(base, "mod generated_compile;").expect("Write failed");
}

fn generate_test_for_file(
    name: &str,
    modules: &mut Vec<String>,
    out: &Path,
    include_file: &Path,
) -> io::Result<()> {
    let basename = name
        .strip_suffix(".go")
        .expect("We checked the suffix earlier");
    modules.push(basename.to_string());
    let output_path = out.with_file_name(basename).with_extension("rs");
    let mut file = File::create(&output_path)?;
    writeln!(file, "use gor::test::test_go_file;")?;
    writeln!(file)?;
    writeln!(file, "#[tokio::test]")?;
    writeln!(file, "async fn go_file() {{")?;
    writeln!(file, "    test_go_file({:?}).await;", include_file)?;
    writeln!(file, "}}")?;

    Ok(())
}
