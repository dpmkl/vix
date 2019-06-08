use std::fmt;
use std::fmt::Write;
use termion;
use termion::color;
use xrl::Style;

fn get_color(argb_color: u32) -> color::Rgb {
    let r = ((argb_color & 0x00ff_0000) >> 16) as u8;
    let g = ((argb_color & 0x0000_ff00) >> 8) as u8;
    let b = (argb_color & 0x0000_00ff) as u8;
    color::Rgb(r, g, b)
}

pub fn set_style(style: &Style) -> Result<String, fmt::Error> {
    if style.id == 0 {
        return Ok(format!("{}", termion::style::Invert));
    }

    let mut s = String::new();

    if let Some(fg_color) = style.fg_color {
        write!(&mut s, "{}", color::Fg(get_color(fg_color)))?;
    }
    if let Some(bg_color) = style.bg_color {
        if bg_color != 0 {
            write!(&mut s, "{}", color::Bg(get_color(bg_color)))?;
        }
    }
    if let Some(italic) = style.italic {
        if italic {
            write!(&mut s, "{}", termion::style::Italic)?;
        }
    }
    if let Some(underline) = style.underline {
        if underline {
            write!(&mut s, "{}", termion::style::Underline)?;
        }
    }
    Ok(s)
}

pub fn reset_style(style: &Style) -> Result<String, fmt::Error> {
    if style.id == 0 {
        return Ok(format!("{}", termion::style::NoInvert));
    }

    let mut s = String::new();

    if style.fg_color.is_some() {
        write!(&mut s, "{}", color::Fg(color::Reset))?;
    }
    if let Some(bg_color) = style.bg_color {
        if bg_color != 0 {
            write!(&mut s, "{}", color::Bg(get_color(bg_color)))?;
        }
    }
    if let Some(italic) = style.italic {
        if italic {
            write!(&mut s, "{}", termion::style::Italic)?;
        }
    }
    if let Some(underline) = style.underline {
        if underline {
            write!(&mut s, "{}", termion::style::Underline)?;
        }
    }
    Ok(s)
}
