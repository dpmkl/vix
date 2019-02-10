use termion::clear::{All, CurrentLine};
use termion::color::{self, AnsiValue, Bg, Fg};
use termion::cursor::{Goto, Hide};
use termion::style::{Bold, Reset, Underline};

fn get_colors(bg: AnsiValue, fg: AnsiValue) -> (Bg<AnsiValue>, Fg<AnsiValue>) {
    (Bg(bg), Fg(fg))
}

fn bar(len: usize) -> String {
    format!("+{}+\n", "-".repeat(len + 4))
}

fn hdr(hdr: &str, len: usize) -> String {
    let (bg, fg) = get_colors(AnsiValue::rgb(0x5, 0x0, 0x0), AnsiValue::rgb(0x0, 0x0, 0x0));
    format!(
        "|{}{}{}{}{}{} {}   |\n",
        Bold,
        Underline,
        hdr,
        Reset,
        bg,
        fg,
        " ".repeat(len - hdr.len())
    )
}

fn line(txt: &str, len: usize) -> String {
    format!("|  {}{}  |\n", txt, " ".repeat(len))
}

fn dialog(title: &str, msg: &str, win_size: u16) -> String {
    let lines: Vec<&str> = msg.split("\n").collect();
    let len = lines.iter().max_by_key(|l| l.len()).unwrap().len() as u16 + 6;
    let height = lines.len() + 3;
    let mut lines: Vec<String> = lines
        .iter()
        .map(|x| line(x, len as usize - x.len()))
        .collect();

    let mut top = (win_size / 2) - (height / 2) as u16;
    let left = 40 - (len / 2) as u16;
    let mut i = 0;
    let mut res = String::new();
    res += &format!("{}", Hide);
    let (bg, fg) = get_colors(AnsiValue::rgb(0x5, 0x0, 0x0), AnsiValue::rgb(0x0, 0x0, 0x0));

    lines.insert(0, bar(len as usize));
    lines.insert(1, hdr(&format!("Error: {}", title), len as usize));
    lines.push(bar(len as usize).clone());
    top += 1;
    for l in lines {
        res += &format!(
            "{}{}{}{}{}\n",
            Goto(left, top + i as u16),
            bg,
            fg,
            l,
            Goto(left + len as u16, top - (height / 2) as u16)
        );
        i += 1;
    }
    res
}

pub fn error_dialog(msg: &str, win_size: u16) -> String {
    format!(
        "{}{}{}",
        Reset,
        dialog(
            "std::io::Error",
            "Could not open file.\naqwe;lfkasldkfja;sd\nasfd",
            win_size
        ),
        Reset
    )
}
