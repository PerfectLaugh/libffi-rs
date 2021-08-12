use crate::common::*;

pub fn build_and_link() {
    let out_dir = env::var("OUT_DIR").unwrap();
    let build_dir = Path::new(&out_dir).join("libffi-build");
    let prefix = Path::new(&out_dir).join("libffi-root");

    let mut libdir = Path::new(&prefix).join("lib");

    let target = std::env::var("TARGET").unwrap();
    if target.starts_with("i686-") {
        libdir = Path::new(&prefix).join("lib32");
    }

    // Copy LIBFFI_DIR into build_dir to avoid an unnecessary build
    if let Err(e) = fs::remove_dir_all(&build_dir) {
        assert_eq!(
            e.kind(),
            std::io::ErrorKind::NotFound,
            "can't remove the build directory: {}",
            e
        );
    }
    run_command(
        "Copying libffi into the build directory",
        Command::new("cp").arg("-R").arg("libffi").arg(&build_dir),
    );

    // Generate configure, run configure, make, make install
    configure_libffi(prefix, &build_dir);

    run_command(
        "Building libffi",
        make_cmd::make()
            .env_remove("DESTDIR")
            .arg("install")
            .current_dir(&build_dir),
    );

    // Cargo linking directives
    println!("cargo:rustc-link-lib=static=ffi");
    println!("cargo:rustc-link-search={}", libdir.display());
}

pub fn probe_and_link() {
    println!("cargo:rustc-link-lib=dylib=ffi");
}

pub fn configure_libffi(prefix: PathBuf, build_dir: &Path) {
    let mut command = Command::new("sh");

    command
        .arg("configure")
        .arg("--with-pic")
        .arg("--disable-docs");

    let target = std::env::var("TARGET").unwrap();
    if target != std::env::var("HOST").unwrap() {
        command.arg(format!("--host={}", target.to_string()));
    }
    if target.starts_with("i686-") {
        command.arg("CFLAGS=-m32");
        command.arg("CXXFLAGS=-m32");
        command.arg("LDFLAGS=-m32");
    }

    command.current_dir(&build_dir);

    if cfg!(windows) {
        // When using MSYS2, OUT_DIR will be a Windows like path such as
        // C:\foo\bar. Unfortunately, the various scripts used for building
        // libffi do not like such a path, so we have to turn this into a Unix
        // like path such as /c/foo/bar.
        //
        // This code assumes the path only uses : for the drive letter, and only
        // uses \ as a component separator. It will likely break for file paths
        // that include a :.
        let mut msys_prefix = prefix
            .to_str()
            .unwrap()
            .replace(":\\", "/")
            .replace("\\", "/");

        msys_prefix.insert(0, '/');

        command.arg("--prefix").arg(msys_prefix);
    } else {
        command.arg("--prefix").arg(prefix);
    }

    run_command("Configuring libffi", &mut command);
}
