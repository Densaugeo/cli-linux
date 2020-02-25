use std::io::{Read, Write};
use nix::sys::{termios, signal};

fn terminal_size() -> std::io::Result<(u16, u16)> {
    let mut winsize = libc::winsize { ws_row: 0, ws_col: 0, ws_xpixel: 0, ws_ypixel: 0 };
    
    match unsafe { libc::ioctl(libc::STDOUT_FILENO, libc::TIOCGWINSZ, &mut winsize) } {
        0 => Ok((winsize.ws_col as u16, winsize.ws_row as u16)),
        _ => Err(std::io::Error::last_os_error())
    }
}

extern "C" fn resize_handler(_a: i32) {
    std::thread::sleep(std::time::Duration::from_millis(100));
    let (cols, rows) = match terminal_size() {
        Ok(v) => v,
        Err(e) => { eprintln!("\x1B[2HError reading terminal size: {}", e); return }
    };
    print!("\x1B[2J\x1B[1;1H"); // Clear screen, move to top left NOTE WSL needed a hide cursor here
    print!("{}", "#".repeat(cols as usize));
    for _ in 1..rows - 1 {
        print!("#\x1B[{}C#", cols - 2);
    }
    print!("{}", "#".repeat(cols as usize)); // NOTE WSL had trouble with the # in the bottom left corner
    print!("\x1B[{};{}H{} x {}", rows/2, cols/2 - 3, cols, rows);
    std::io::stdout().flush().unwrap();
}

fn main() {
    let resize_action_nix = signal::SigAction::new(
        signal::SigHandler::Handler(resize_handler),
        signal::SaFlags::empty(),
        signal::SigSet::empty()
    );
    unsafe { signal::sigaction(signal::Signal::SIGWINCH, &resize_action_nix) }.expect(
        "Error creating resize handler");
    
    let old_termios = termios::tcgetattr(libc::STDIN_FILENO).expect("Error reading terminal settings");
    let mut new_termios = old_termios.clone();
    new_termios.local_flags &= !(termios::LocalFlags::ICANON | termios::LocalFlags::ECHO);
    termios::tcsetattr(libc::STDIN_FILENO, termios::SetArg::TCSANOW, &new_termios).expect(
        "Error changing terminal settings");
    
    print!("\x1B[?1049h\x1B[?25l"); // Activate alternate screen buffer, hide cursor
    std::io::stdout().flush().expect("Error flushing output");
    //println!("\x1B[33mHello,\x1B[0m world!");
    //println!("\x1B[38;2;255;0;0;mHello,\x1B[0m\x1b[999D\x1b[12Cworld!");
    resize_handler(0);
    
    for byte in std::io::stdin().bytes() {
        match byte {
            Ok(97) => println!("AAAAAA!"),
            Ok(98) => println!("{:?}", terminal_size()),
            Ok(99) => resize_handler(0),
            Ok(10) => break,
            Ok(byte) => println!("You have hit: {:?}", byte),
            Err(_) => ()
        }
    }
    
    termios::tcsetattr(libc::STDIN_FILENO, termios::SetArg::TCSANOW, &old_termios).expect(
        "Error restoring terminal settings");
    print!("\x1B[?1049l"); // Deactivate alternate screen buffer
    std::io::stdout().flush().expect("Error flushing output");
}
