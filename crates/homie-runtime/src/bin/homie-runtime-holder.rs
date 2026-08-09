use homie_runtime::Exit;
use homie_runtime::holder::{HolderRequest, HolderResponse};
use homie_runtime::{Pty, PtySpec};
use homie_runtime::{kill_process_tree, process_tree};
use std::io::{BufRead, BufReader, Read, Write};
use std::os::unix::net::UnixListener;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::Duration;

fn main() -> std::io::Result<()> {
    let args = Args::parse()?;
    if let Some(parent) = args.socket.parent() {
        std::fs::create_dir_all(parent)?;
    }
    if let Some(parent) = args.log_path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let _ = std::fs::remove_file(&args.socket);
    let _ = std::fs::remove_file(&args.status_file);

    let path = std::env::var("PATH").unwrap_or_else(|_| "/usr/bin:/bin".to_string());
    let spec = PtySpec::new(
        vec!["/bin/sh".to_string(), "-i".to_string()],
        args.cwd.clone(),
    )
    .size(args.cols, args.rows)
    .env("TERM", "xterm-256color")
    .env("PATH", &path);
    let pty = Pty::spawn(&spec)?;
    let reader = pty.reader()?;
    let pty = Arc::new(Mutex::new(pty));
    let geometry = Arc::new(Mutex::new((args.cols, args.rows)));
    let epoch_offset = log_offset(&args.log_path).unwrap_or(0);

    std::fs::write(&args.pid_file, format!("{}\n", std::process::id()))?;
    write_status(&args.status_file, "running")?;
    let log_path = args.log_path.clone();
    let reader_thread = std::thread::spawn(move || pump_output(reader, log_path));
    let listener = UnixListener::bind(&args.socket)?;

    for stream in listener.incoming() {
        let Ok(stream) = stream else {
            break;
        };
        let terminate = serve(stream, &pty, &geometry, &args.log_path, epoch_offset);
        if terminate {
            break;
        }
    }

    if let Ok(mut pty) = pty.lock() {
        let _ = kill_process_tree(pty.pid() as i32, Duration::from_millis(500));
        let status = match pty.terminate(Duration::from_millis(100)) {
            Ok(exit) => exit_status(exit),
            Err(error) => format!("exited_error_{error}"),
        };
        let _ = write_status(&args.status_file, &status);
    }
    let _ = reader_thread.join();
    let _ = std::fs::remove_file(&args.socket);
    let _ = std::fs::remove_file(&args.pid_file);
    Ok(())
}

fn serve(
    mut stream: std::os::unix::net::UnixStream,
    pty: &Arc<Mutex<Pty>>,
    geometry: &Arc<Mutex<(u16, u16)>>,
    log_path: &PathBuf,
    epoch_offset: u64,
) -> bool {
    let mut line = String::new();
    let response =
        match BufReader::new(stream.try_clone().expect("clone stream")).read_line(&mut line) {
            Ok(_) => match serde_json::from_str::<HolderRequest>(&line) {
                Ok(HolderRequest::Write { text, submit }) => {
                    let result = write_pty(pty, &text, submit);
                    match result {
                        Ok(()) => HolderResponse {
                            ok: true,
                            error: None,
                            pid: None,
                            status: None,
                            tree_size: None,
                            cols: None,
                            rows: None,
                            log_offset: None,
                            epoch_offset: None,
                        },
                        Err(error) => HolderResponse {
                            ok: false,
                            error: Some(error.to_string()),
                            pid: None,
                            status: None,
                            tree_size: None,
                            cols: None,
                            rows: None,
                            log_offset: None,
                            epoch_offset: None,
                        },
                    }
                }
                Ok(HolderRequest::WriteBytes { bytes }) => {
                    let result = write_pty_bytes(pty, &bytes);
                    match result {
                        Ok(()) => HolderResponse {
                            ok: true,
                            error: None,
                            pid: None,
                            status: None,
                            tree_size: None,
                            cols: None,
                            rows: None,
                            log_offset: None,
                            epoch_offset: None,
                        },
                        Err(error) => HolderResponse {
                            ok: false,
                            error: Some(error.to_string()),
                            pid: None,
                            status: None,
                            tree_size: None,
                            cols: None,
                            rows: None,
                            log_offset: None,
                            epoch_offset: None,
                        },
                    }
                }
                Ok(HolderRequest::Resize { cols, rows }) => {
                    let result = resize_pty(pty, geometry, cols, rows);
                    match result {
                        Ok(()) => HolderResponse {
                            ok: true,
                            error: None,
                            pid: None,
                            status: Some("resized".to_string()),
                            tree_size: None,
                            cols: Some(cols),
                            rows: Some(rows),
                            log_offset: log_offset(log_path).ok(),
                            epoch_offset: Some(epoch_offset),
                        },
                        Err(error) => HolderResponse {
                            ok: false,
                            error: Some(error.to_string()),
                            pid: None,
                            status: None,
                            tree_size: None,
                            cols: None,
                            rows: None,
                            log_offset: None,
                            epoch_offset: None,
                        },
                    }
                }
                Ok(HolderRequest::Stat) => holder_stat(pty, geometry, log_path, epoch_offset),
                Ok(HolderRequest::KillTree) => {
                    let result = kill_tree(pty);
                    match result {
                        Ok(tree_size) => HolderResponse {
                            ok: true,
                            error: None,
                            pid: None,
                            status: Some("kill_tree_sent".to_string()),
                            tree_size: Some(tree_size),
                            cols: None,
                            rows: None,
                            log_offset: log_offset(log_path).ok(),
                            epoch_offset: Some(epoch_offset),
                        },
                        Err(error) => HolderResponse {
                            ok: false,
                            error: Some(error.to_string()),
                            pid: None,
                            status: None,
                            tree_size: None,
                            cols: None,
                            rows: None,
                            log_offset: None,
                            epoch_offset: None,
                        },
                    }
                }
                Ok(HolderRequest::Terminate) => {
                    let response = HolderResponse {
                        ok: true,
                        error: None,
                        pid: None,
                        status: Some("terminating".to_string()),
                        tree_size: None,
                        cols: None,
                        rows: None,
                        log_offset: log_offset(log_path).ok(),
                        epoch_offset: Some(epoch_offset),
                    };
                    let _ = write_response(&mut stream, &response);
                    return true;
                }
                Err(error) => HolderResponse {
                    ok: false,
                    error: Some(error.to_string()),
                    pid: None,
                    status: None,
                    tree_size: None,
                    cols: None,
                    rows: None,
                    log_offset: None,
                    epoch_offset: None,
                },
            },
            Err(error) => HolderResponse {
                ok: false,
                error: Some(error.to_string()),
                pid: None,
                status: None,
                tree_size: None,
                cols: None,
                rows: None,
                log_offset: None,
                epoch_offset: None,
            },
        };
    let _ = write_response(&mut stream, &response);
    false
}

fn holder_stat(
    pty: &Arc<Mutex<Pty>>,
    geometry: &Arc<Mutex<(u16, u16)>>,
    log_path: &PathBuf,
    epoch_offset: u64,
) -> HolderResponse {
    let Ok(mut pty) = pty.lock() else {
        return HolderResponse {
            ok: false,
            error: Some("holder pty lock unavailable".to_string()),
            pid: None,
            status: None,
            tree_size: None,
            cols: None,
            rows: None,
            log_offset: None,
            epoch_offset: None,
        };
    };
    let pid = pty.pid();
    let tree_size = Some(process_tree(pid as i32).len());
    let (cols, rows) = *geometry.lock().expect("geometry");
    let log_offset = log_offset(log_path).ok();
    match pty.try_wait() {
        Ok(None) => HolderResponse {
            ok: true,
            error: None,
            pid: Some(pid),
            status: Some("running".to_string()),
            tree_size,
            cols: Some(cols),
            rows: Some(rows),
            log_offset,
            epoch_offset: Some(epoch_offset),
        },
        Ok(Some(exit)) => HolderResponse {
            ok: true,
            error: None,
            pid: Some(pid),
            status: Some(exit_status(exit)),
            tree_size,
            cols: Some(cols),
            rows: Some(rows),
            log_offset,
            epoch_offset: Some(epoch_offset),
        },
        Err(error) => HolderResponse {
            ok: false,
            error: Some(error.to_string()),
            pid: Some(pid),
            status: None,
            tree_size,
            cols: Some(cols),
            rows: Some(rows),
            log_offset,
            epoch_offset: Some(epoch_offset),
        },
    }
}

fn kill_tree(pty: &Arc<Mutex<Pty>>) -> std::io::Result<usize> {
    let pty = pty.lock().expect("pty");
    let tree = process_tree(pty.pid() as i32);
    kill_process_tree(pty.pid() as i32, Duration::from_millis(500))?;
    Ok(tree.len())
}

fn exit_status(exit: Exit) -> String {
    match exit {
        Exit::Code(0) => "exited".to_string(),
        Exit::Code(code) => format!("exited_code_{code}"),
        Exit::Signal(signal) => format!("exited_signal_{signal}"),
    }
}

fn write_status(path: &PathBuf, status: &str) -> std::io::Result<()> {
    std::fs::write(path, format!("{status}\n"))
}

fn write_response(
    stream: &mut std::os::unix::net::UnixStream,
    response: &HolderResponse,
) -> std::io::Result<()> {
    let mut encoded = serde_json::to_vec(response)?;
    encoded.push(b'\n');
    stream.write_all(&encoded)
}

fn write_pty(pty: &Arc<Mutex<Pty>>, text: &str, submit: bool) -> std::io::Result<()> {
    let mut writer = pty.lock().expect("pty").writer()?;
    writer.write_all(text.as_bytes())?;
    if submit {
        writer.write_all(b"\n")?;
    }
    writer.flush()
}

fn write_pty_bytes(pty: &Arc<Mutex<Pty>>, bytes: &[u8]) -> std::io::Result<()> {
    let mut writer = pty.lock().expect("pty").writer()?;
    writer.write_all(bytes)?;
    writer.flush()
}

fn resize_pty(
    pty: &Arc<Mutex<Pty>>,
    geometry: &Arc<Mutex<(u16, u16)>>,
    cols: u16,
    rows: u16,
) -> std::io::Result<()> {
    if cols < 2 || rows < 2 {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "resize requires cols/rows >= 2",
        ));
    }
    pty.lock().expect("pty").resize(cols, rows)?;
    *geometry.lock().expect("geometry") = (cols, rows);
    Ok(())
}

fn log_offset(path: &PathBuf) -> std::io::Result<u64> {
    match std::fs::metadata(path) {
        Ok(metadata) => Ok(metadata.len()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(0),
        Err(error) => Err(error),
    }
}

fn pump_output(mut reader: homie_runtime::PtyStream, path: PathBuf) {
    let mut file = match std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
    {
        Ok(file) => file,
        Err(_) => return,
    };
    let mut buffer = [0_u8; 8192];
    loop {
        match reader.read(&mut buffer) {
            Ok(0) => break,
            Ok(n) => {
                if file.write_all(&buffer[..n]).is_err() {
                    break;
                }
                let _ = file.flush();
            }
            Err(error) if error.kind() == std::io::ErrorKind::Interrupted => continue,
            Err(_) => break,
        }
    }
}

struct Args {
    socket: PathBuf,
    pid_file: PathBuf,
    status_file: PathBuf,
    log_path: PathBuf,
    cwd: PathBuf,
    cols: u16,
    rows: u16,
}

impl Args {
    fn parse() -> std::io::Result<Self> {
        let mut socket = None;
        let mut pid_file = None;
        let mut status_file = None;
        let mut log_path = None;
        let mut cwd = None;
        let mut cols = 120;
        let mut rows = 40;
        let mut args = std::env::args().skip(1);
        while let Some(arg) = args.next() {
            match arg.as_str() {
                "--socket" => socket = args.next().map(PathBuf::from),
                "--pid-file" => pid_file = args.next().map(PathBuf::from),
                "--status-file" => status_file = args.next().map(PathBuf::from),
                "--log-path" => log_path = args.next().map(PathBuf::from),
                "--cwd" => cwd = args.next().map(PathBuf::from),
                "--cols" => {
                    cols = args
                        .next()
                        .and_then(|value| value.parse().ok())
                        .unwrap_or(cols)
                }
                "--rows" => {
                    rows = args
                        .next()
                        .and_then(|value| value.parse().ok())
                        .unwrap_or(rows)
                }
                _ => {}
            }
        }
        Ok(Self {
            socket: socket.ok_or_else(|| invalid("--socket"))?,
            pid_file: pid_file.ok_or_else(|| invalid("--pid-file"))?,
            status_file: status_file.ok_or_else(|| invalid("--status-file"))?,
            log_path: log_path.ok_or_else(|| invalid("--log-path"))?,
            cwd: cwd.ok_or_else(|| invalid("--cwd"))?,
            cols,
            rows,
        })
    }
}

fn invalid(arg: &str) -> std::io::Error {
    std::io::Error::new(
        std::io::ErrorKind::InvalidInput,
        format!("missing required argument {arg}"),
    )
}
