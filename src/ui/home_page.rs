use crate::board;
use mousefood::prelude::*;
use mousefood::ratatui::widgets::{Block, Paragraph, Wrap};
use mousefood::{fonts, EmbeddedBackend};
use ssd1680::prelude::Display;

pub fn mouse_food_test(board: &mut board::BoardPeripherals) -> anyhow::Result<()> {
    {
        let config = EmbeddedBackendConfig {
            font_regular: fonts::MONO_6X13,
            ..Default::default()
        };
        let backend = EmbeddedBackend::new(&mut board.bw_buf, config);
        let mut terminal = Terminal::new(backend)?;
        terminal.draw(|f| {
            // let custom_area = Rect::new(0, 0, 296, 128);
            let text = "Ratatui on embedded devices!";
            let paragraph = Paragraph::new(text.dark_gray()).wrap(Wrap { trim: true });
            let bordered_block = Block::bordered()
                .border_style(Style::new().yellow())
                .title("Mousefood");
            f.render_widget(paragraph.block(bordered_block), f.area());
        })?;
    }
    board
        .ssd1680
        .update_bw_frame(board.bw_buf.buffer())
        .unwrap();
    board.ssd1680.display_frame(&mut board.delay).unwrap();
    Ok(())
}
