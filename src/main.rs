use netstring::ReadNetstring;
use netstring::Shutdown;
use std::io::Error;
use std::io::ErrorKind::ConnectionAborted;
use std::io::ErrorKind::Unsupported;
use std::io::Read;
use std::io::Result;
use std::io::StdinLock;
use std::io::stdin;
use std::net::Shutdown as ShutdownMode;

struct ShutdownableStdin(StdinLock<'static>);

impl Read for ShutdownableStdin {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        self.0.read(buf)
    }
}

impl Shutdown for ShutdownableStdin {
    fn shutdown(&self, _how: ShutdownMode) -> Result<()> {
        Err(Error::from(Unsupported))
    }
}

fn main() -> Result<()> {
    let mut stdin = ShutdownableStdin(stdin().lock());

    loop {
        match stdin.read_netstring() {
            Ok(str) => { println!("{}", str); }
            Err(err) => {
                return match err.kind() {
                    ConnectionAborted => Ok(()),
                    _ => Err(err),
                };
            }
        }
    }
}
