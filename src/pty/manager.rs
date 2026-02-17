use anyhow::{Context, Result};
use std::os::fd::{AsRawFd, FromRawFd, OwnedFd, RawFd};

/// Async wrapper around a raw PTY fd using tokio's AsyncFd.
pub struct AsyncPtyFd(tokio::io::unix::AsyncFd<OwnedFd>);

impl AsyncPtyFd {
    fn new(fd: OwnedFd) -> Result<Self> {
        // Set non-blocking
        let raw = fd.as_raw_fd();
        let flags = unsafe { libc::fcntl(raw, libc::F_GETFL) };
        if flags < 0 {
            return Err(std::io::Error::last_os_error()).context("fcntl F_GETFL");
        }
        let ret = unsafe { libc::fcntl(raw, libc::F_SETFL, flags | libc::O_NONBLOCK) };
        if ret < 0 {
            return Err(std::io::Error::last_os_error()).context("fcntl F_SETFL O_NONBLOCK");
        }
        Ok(Self(tokio::io::unix::AsyncFd::new(fd)?))
    }

    fn as_raw_fd(&self) -> RawFd {
        self.0.get_ref().as_raw_fd()
    }

    pub async fn read(&self, buf: &mut [u8]) -> std::io::Result<usize> {
        loop {
            let mut guard = self.0.readable().await?;
            match guard.try_io(|inner| {
                let fd = inner.get_ref().as_raw_fd();
                let n = unsafe {
                    libc::read(fd, buf.as_mut_ptr() as *mut libc::c_void, buf.len())
                };
                if n < 0 {
                    Err(std::io::Error::last_os_error())
                } else {
                    Ok(n as usize)
                }
            }) {
                Ok(result) => return result,
                Err(_would_block) => continue,
            }
        }
    }

    async fn write_all(&self, data: &[u8]) -> std::io::Result<()> {
        let mut written = 0;
        while written < data.len() {
            let mut guard = self.0.writable().await?;
            match guard.try_io(|inner| {
                let fd = inner.get_ref().as_raw_fd();
                let n = unsafe {
                    libc::write(
                        fd,
                        data[written..].as_ptr() as *const libc::c_void,
                        data.len() - written,
                    )
                };
                if n < 0 {
                    Err(std::io::Error::last_os_error())
                } else {
                    Ok(n as usize)
                }
            }) {
                Ok(Ok(n)) => written += n,
                Ok(Err(e)) => return Err(e),
                Err(_would_block) => continue,
            }
        }
        Ok(())
    }
}

pub struct PtyManager {
    writer: AsyncPtyFd,
    master_raw: RawFd,
    _child: tokio::process::Child,
}

impl PtyManager {
    pub fn spawn(cols: u16, rows: u16) -> Result<(Self, AsyncPtyFd)> {
        let cols = cols.max(2);
        let rows = rows.max(2);

        // Open PTY pair using nix::openpty with initial size
        let result = nix::pty::openpty(
            Some(&nix::pty::Winsize {
                ws_row: rows,
                ws_col: cols,
                ws_xpixel: 0,
                ws_ypixel: 0,
            }),
            None,
        )
        .context("openpty")?;

        let master_fd: OwnedFd = result.master;
        let slave_fd: OwnedFd = result.slave;
        let slave_raw = slave_fd.as_raw_fd();
        let master_raw = master_fd.as_raw_fd();

        // dup the master fd: one copy for reading, one for writing
        let reader_raw = unsafe { libc::dup(master_raw) };
        if reader_raw < 0 {
            return Err(std::io::Error::last_os_error()).context("dup master fd for reader");
        }
        let reader_fd = unsafe { OwnedFd::from_raw_fd(reader_raw) };

        // Spawn child with pre_exec to set up slave as controlling TTY
        let mut cmd = tokio::process::Command::new("claude");
        cmd.arg("--dangerously-skip-permissions");
        unsafe {
            cmd.pre_exec(move || {
                // Create a new session
                if libc::setsid() < 0 {
                    return Err(std::io::Error::last_os_error());
                }
                // Set the slave as the controlling terminal
                if libc::ioctl(slave_raw, libc::TIOCSCTTY as _, 0i32) < 0 {
                    return Err(std::io::Error::last_os_error());
                }
                // Dup slave to stdin/stdout/stderr
                libc::dup2(slave_raw, 0);
                libc::dup2(slave_raw, 1);
                libc::dup2(slave_raw, 2);
                if slave_raw > 2 {
                    libc::close(slave_raw);
                }
                Ok(())
            });
        }

        let child = cmd.spawn().context("spawn claude (is `claude` in PATH?)")?;

        // Close the slave fd in the parent — child has its own copies after fork
        drop(slave_fd);

        // Wrap fds for async I/O
        let writer = AsyncPtyFd::new(master_fd).context("async wrap master writer fd")?;
        let reader = AsyncPtyFd::new(reader_fd).context("async wrap master reader fd")?;

        Ok((
            Self {
                master_raw: writer.as_raw_fd(),
                writer,
                _child: child,
            },
            reader,
        ))
    }

    pub fn master_raw_fd(&self) -> RawFd {
        self.master_raw
    }

    pub async fn write_input(&self, data: &[u8]) -> Result<()> {
        self.writer.write_all(data).await?;
        Ok(())
    }

    pub async fn inject_input(&self, text: &str) -> Result<()> {
        self.writer.write_all(text.as_bytes()).await?;
        Ok(())
    }

    pub fn resize(&self, cols: u16, rows: u16) -> Result<()> {
        let ws = libc::winsize {
            ws_row: rows,
            ws_col: cols,
            ws_xpixel: 0,
            ws_ypixel: 0,
        };
        let ret = unsafe { libc::ioctl(self.master_raw, libc::TIOCSWINSZ, &ws) };
        if ret < 0 {
            return Err(std::io::Error::last_os_error()).context("TIOCSWINSZ resize");
        }
        Ok(())
    }
}

pub async fn read_pty_loop(
    reader: AsyncPtyFd,
    tx: tokio::sync::mpsc::UnboundedSender<crate::event::AppEvent>,
) {
    let mut buf = vec![0u8; 4096];
    loop {
        match reader.read(&mut buf).await {
            Ok(0) | Err(_) => {
                let _ = tx.send(crate::event::AppEvent::PtyExited);
                break;
            }
            Ok(n) => {
                let _ = tx.send(crate::event::AppEvent::PtyOutput(buf[..n].to_vec()));
            }
        }
    }
}
