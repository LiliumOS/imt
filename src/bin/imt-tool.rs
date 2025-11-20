use std::{
    io::ErrorKind,
    process::{Command, ExitCode, Stdio},
};

use bincode::error::DecodeError;
use imt::bundle::{Bundle, Path};

fn main() -> ExitCode {
    let mut args = std::env::args();
    let prg_name = args.next().unwrap();
    match real_main(&prg_name, args) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("{prg_name}: {e}");
            ExitCode::FAILURE
        }
    }
}

fn real_main(prg_name: &str, mut args: impl Iterator<Item = String>) -> std::io::Result<()> {
    let mut children = Vec::new();

    let mut input = Vec::new();

    let mut is_bundle = false;
    let mut unzip_prg = None;
    let mut prefix = None;

    while let Some(arg) = args.next() {
        match &*arg {
            "--version" => {
                println!("imt-tool v{}", core::env!("CARGO_PKG_VERSION"));
                return Ok(());
            }
            "--help" => {
                println!("Usage: {prg_name} [OPTIONS...] [--] [file..]");
                println!("Displays contents of IMT files or IMT Bundle files");
                println!("If no file is provided, read from standard input");
                println!("Options:");
                println!("\t--help: Print this message, and exit");
                println!("\t--version: Print version information and exit");
                println!("\t--bundle: Treats the input file as a TAR archives containing a bundle");
                println!("\t--prefix <path>: treats the files as if it starts in module <path>");
                println!(
                    "\t--unzip <prg>: Processes each input file through <prg> (e.g. gzip/xz/lzma - expects the command to follow gzip CLI)"
                );
                return Ok(());
            }
            "--bundle" => {
                is_bundle = true;
            }
            "--prefix" => {
                prefix = Some(args.next().ok_or_else(|| {
                    std::io::Error::new(ErrorKind::InvalidInput, "--prefix requires and argument")
                })?);
            }
            "--unzip" => {
                unzip_prg = Some(args.next().ok_or_else(|| {
                    std::io::Error::new(ErrorKind::InvalidInput, "--unzip requires and argument")
                })?);
            }
            "--" => break,
            x if x.starts_with("--") => {
                println!("{prg_name}:")
            }
            _ => {
                input.push(arg);
                break;
            }
        }
    }

    input.extend(args);

    let prefix = prefix
        .map(|prefix| Path(prefix.split("::").map(str::to_string).collect()))
        .unwrap_or_else(|| Path(vec![]));

    let mut bundle = Bundle::create();

    if let Some(unzip_prg) = &unzip_prg {
        let mut files = Vec::new();
        if input.is_empty() {
            let mut child = Command::new(&unzip_prg)
                .arg("-d")
                .stdout(Stdio::piped())
                .spawn()?;
            files.push((Path(vec![]), child.stdout.take().unwrap()));
            children.push(child);
        } else {
            for input in &input {
                let path = std::path::Path::new(input);
                let name = path
                    .file_stem()
                    .ok_or_else(|| {
                        std::io::Error::new(
                            ErrorKind::IsADirectory,
                            "input files must be files, not directories",
                        )
                    })?
                    .to_str()
                    .unwrap();
                let file = std::fs::File::open(path)?;
                let mut child = Command::new(&unzip_prg)
                    .arg("-d")
                    .stdin(file)
                    .stdout(Stdio::piped())
                    .spawn()?;

                files.push((Path(vec![name.to_string()]), child.stdout.take().unwrap()));
                children.push(child);
            }
        }

        for (name, output) in files {
            if is_bundle {
                #[cfg(not(feature = "tar"))]
                {
                    return Err(std::io::Error::new(
                        ErrorKind::Other,
                        "--bundle requires building with the tar feature",
                    ));
                }
                #[cfg(feature = "tar")]
                {
                    bundle
                        .parse_tar(prefix.clone(), output)
                        .map_err(|e| match e {
                            DecodeError::Io { inner, .. } => inner,
                            e => std::io::Error::new(ErrorKind::InvalidData, e),
                        })?;
                }
            } else {
                bundle.parse_file(name, output).map_err(|e| match e {
                    DecodeError::Io { inner, .. } => inner,
                    e => std::io::Error::new(ErrorKind::InvalidData, e),
                })?
            }
        }
    } else {
        match (is_bundle, input.is_empty()) {
            #[cfg(not(feature = "tar"))]
            (true, _) => {
                return Err(std::io::Error::new(
                    ErrorKind::Other,
                    "--bundle requires building with the tar feature",
                ));
            }
            #[cfg(feature = "tar")]
            (true, true) => {
                bundle
                    .parse_tar(prefix.clone(), std::io::stdin().lock())
                    .map_err(|e| match e {
                        DecodeError::Io { inner, .. } => inner,
                        e => std::io::Error::new(ErrorKind::InvalidData, e),
                    })?;
            }
            (false, true) => {
                bundle
                    .parse_file(Path(vec![]), std::io::stdin().lock())
                    .map_err(|e| match e {
                        DecodeError::Io { inner, .. } => inner,
                        e => std::io::Error::new(ErrorKind::InvalidData, e),
                    })?;
            }
            #[cfg(feature = "tar")]
            (true, false) => {
                for input in &input {
                    bundle
                        .parse_tar(prefix.clone(), std::fs::File::open(input)?)
                        .map_err(|e| match e {
                            DecodeError::Io { inner, .. } => inner,
                            e => std::io::Error::new(ErrorKind::InvalidData, e),
                        })?;
                }
            }
            (false, false) => {
                for input in &input {
                    let path = std::path::Path::new(input);
                    let name = path
                        .file_stem()
                        .ok_or_else(|| {
                            std::io::Error::new(
                                ErrorKind::IsADirectory,
                                "input files must be files, not directories",
                            )
                        })?
                        .to_str()
                        .unwrap();
                    let file = std::fs::File::open(path)?;
                    bundle
                        .parse_file(Path(vec![name.to_string()]), file)
                        .map_err(|e| match e {
                            DecodeError::Io { inner, .. } => inner,
                            e => std::io::Error::new(ErrorKind::InvalidData, e),
                        })?;
                }
            }
        }
    }

    println!("bundle: {bundle:#?}");

    for (i, mut child) in children.into_iter().enumerate() {
        let status = child.wait()?;

        if !status.success() {
            return Err(std::io::Error::new(
                ErrorKind::Other,
                format!(
                    "{}: {} exited with status: {status}",
                    input.get(i).map(String::as_str).unwrap_or("-"),
                    unzip_prg.unwrap()
                ),
            ));
        }
    }

    Ok(())
}
