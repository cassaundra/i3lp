use launchpad::mk2::*;
use launchpad::RGBColor;

#[derive(Clone, Default, PartialEq)]
pub struct LaunchpadBuffer {
    grid: [[RGBColor; 8]; 8],
    top_buttons: [RGBColor; 8],
    right_buttons: [RGBColor; 8],
}

impl LaunchpadBuffer {
    pub fn set(&mut self, location: &Location, color: &RGBColor) {
        use Location::*;

        match location {
            Pad(x, y) => self.grid[*y as usize][*x as usize] = *color,
            Button(c, ButtonSide::Top) => self.top_buttons[*c as usize] = *color,
            Button(c, ButtonSide::Right) => self.right_buttons[*c as usize] = *color,
        }
    }

    pub fn render(&self, launchpad: &mut impl LaunchpadMk2) -> launchpad::Result<()> {
        let mut buffer = Vec::with_capacity(80);

        for y in 0..8 {
            for x in 0..8 {
                buffer.push((Location::Pad(x as u8, y as u8), self.grid[y][x]));
            }
        }

        for i in 0..8 {
            buffer.push((Location::Button(i as u8, ButtonSide::Top), self.top_buttons[i]));
            buffer.push((Location::Button(i as u8, ButtonSide::Right), self.right_buttons[i]));
        }

        launchpad.light_multi_rgb(buffer)
    }
}
