use anyhow::{Context, Result};
use omarchy_lib::protocol::{Frame, Request, Response};
use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::UnixStream;

const SOCKET_PATH: &str = "/run/omarchy/daemon.sock";

pub struct DaemonClient {
    stream: UnixStream,
    reader: BufReader<UnixStream>,
}

impl DaemonClient {
    pub fn connect() -> Result<Self> {
        let stream = UnixStream::connect(SOCKET_PATH)
            .context("daemon not reachable at /run/omarchy/daemon.sock")?;
        let reader = BufReader::new(stream.try_clone()?);
        Ok(Self { stream, reader })
    }

    /// Send a request and read a single Done frame, skipping any progress frames.
    pub fn send_recv(&mut self, req: Request) -> Result<Response> {
        self.write_request(&req)?;
        self.read_done()
    }

    /// Send a request, call `on_progress` for each progress line, return Done response.
    pub fn stream(&mut self, req: Request, mut on_progress: impl FnMut(&str)) -> Result<Response> {
        self.write_request(&req)?;
        loop {
            match self.read_frame()? {
                Frame::Progress { line } => on_progress(&line),
                Frame::Done(resp) => return Ok(resp),
            }
        }
    }

    fn write_request(&mut self, req: &Request) -> Result<()> {
        let mut line = serde_json::to_string(req)?;
        line.push('\n');
        self.stream.write_all(line.as_bytes()).context("write to daemon socket")?;
        Ok(())
    }

    fn read_frame(&mut self) -> Result<Frame> {
        let mut line = String::new();
        self.reader.read_line(&mut line).context("read from daemon socket")?;
        serde_json::from_str(line.trim_end()).context("parse daemon frame")
    }

    fn read_done(&mut self) -> Result<Response> {
        loop {
            match self.read_frame()? {
                Frame::Done(resp) => return Ok(resp),
                Frame::Progress { .. } => {}
            }
        }
    }
}

