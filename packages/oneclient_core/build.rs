use std::{env, path::PathBuf, process::Command};

fn main() {
    let workspace = PathBuf::from(env::var_os("CARGO_MANIFEST_DIR").unwrap()).join("../..");
    let agent = workspace.join("distribution/elyby-preview-compat");
    for path in [
        agent.join("build.py"),
        agent.join("src"),
        agent.join("LICENSE-ASM.txt"),
        workspace.join("LICENSE"),
    ] {
        println!("cargo:rerun-if-changed={}", path.display());
    }
    for name in ["PYTHON", "JAVA_HOME", "JAVAC", "ONEFORALL_ASM_JAR"] {
        println!("cargo:rerun-if-env-changed={name}");
    }
    let python = env::var_os("PYTHON").unwrap_or_else(|| {
        let candidates = if cfg!(windows) { ["python", "python3"] } else { ["python3", "python"] };
        candidates.into_iter()
            .find(|name| Command::new(name).arg("--version").output().is_ok_and(|out| {
                out.status.success() && (String::from_utf8_lossy(&out.stdout).starts_with("Python 3.")
                    || String::from_utf8_lossy(&out.stderr).starts_with("Python 3."))
            }))
            .expect("Building the preview agent requires Python 3. Install it or set PYTHON to its executable path.").into()
    });
    let output = PathBuf::from(env::var_os("OUT_DIR").unwrap()).join("elyby-preview-compat.jar");
    let result = Command::new(python)
        .arg(agent.join("build.py"))
        .arg("--output")
        .arg(&output)
        .output()
        .expect("Could not run the Java preview agent build script");
    if !result.status.success() {
        panic!(
            "Could not compile the preview agent from source. Install Python 3 and JDK 17+ (or set JAVA_HOME/JAVAC).\n{}\n{}",
            String::from_utf8_lossy(&result.stdout),
            String::from_utf8_lossy(&result.stderr)
        );
    }
    assert!(
        output.is_file(),
        "Agent build did not produce {}",
        output.display()
    );
}
