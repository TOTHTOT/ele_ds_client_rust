use mousefood::prelude::{Alignment, Constraint, Frame, Layout, Rect};
use mousefood::ratatui::layout::Flex;
use mousefood::ratatui::widgets::{Block, Clear, Paragraph, Wrap};
use std::default::Default;

#[derive(Default)]
pub struct PopupMsg {
    title: String,
    msg: String,
}
impl PopupMsg {
    /*pub fn build_popup_msg(&self, screen: &mut Screen) -> anyhow::Result<()> {
        {
            let config = EmbeddedBackendConfig {
                font_regular: fonts::MONO_6X13,
                ..Default::default()
            };
            let backend = EmbeddedBackend::new(&mut screen.bw_buf, config);
            let mut terminal = Terminal::new(backend)?;
            terminal.draw(|f| Self::show_popup(f, None, &self.title, &self.msg))?;
        }
        // 单独解构这些, 避免借用问题
        let Screen {
            ref mut ssd1680,
            ref mut delay,
            ref mut bw_buf,
            ..
        } = &mut *screen;
        hw_try!(ssd1680.init(delay), "Ssd1680 init");
        hw_try!(ssd1680.update_bw_frame(bw_buf.buffer()), "Ssd1680 update");
        hw_try!(ssd1680.display_frame(delay), "Ssd1680 display");
        hw_try!(ssd1680.entry_sleep(), "Ssd1680 sleep");
        Ok(())
    }*/

    pub fn new(title: String, msg: String) -> Self {
        Self { title, msg }
    }
    pub fn show_popup(&self, f: &mut Frame, area: Option<Rect>) {
        let block = Block::bordered().title(self.title.as_str());
        let area = area.unwrap_or(f.area());
        let area = popup_area(area, 60, 40);
        let msg_w = Paragraph::new(self.msg.as_str())
            .block(block)
            .alignment(Alignment::Center)
            .wrap(Wrap { trim: true });
        f.render_widget(Clear, area); //this clears out the background
        f.render_widget(msg_w, area);
    }
}

fn popup_area(area: Rect, percent_x: u16, percent_y: u16) -> Rect {
    let vertical = Layout::vertical([Constraint::Percentage(percent_y)]).flex(Flex::Center);
    let horizontal = Layout::horizontal([Constraint::Percentage(percent_x)]).flex(Flex::Center);
    let [area] = vertical.areas(area);
    let [area] = horizontal.areas(area);
    area
}
