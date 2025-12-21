use menu::*;
use std::io::{self, Write};

// 这是一个适配器，让 std::io 能够被嵌入式库使用
pub struct ShellInterface;

impl embedded_io::ErrorType for ShellInterface {
    type Error = embedded_io::ErrorKind;
}

impl embedded_io::Write for ShellInterface {
    fn write(&mut self, buf: &[u8]) -> Result<usize, Self::Error> {
        // 将数据写入真正的 stdout
        io::stdout()
            .write(buf)
            .map_err(|_| embedded_io::ErrorKind::Other)
    }

    fn flush(&mut self) -> Result<(), Self::Error> {
        io::stdout()
            .flush()
            .map_err(|_| embedded_io::ErrorKind::Other)
    }
}

type MyMenuType<'a> = Menu<'a, ShellInterface, ()>;
type MyItemType<'a> = Item<'a, ShellInterface, ()>;

// 注意：参数变为了 5 个，且 _args 类型变为 &[&str]
fn cmd_help(
    _menu: &MyMenuType,
    _item: &MyItemType,
    _args: &[&str], // 修改点
    _interface: &mut ShellInterface, // 新增点
    _context: &mut (),
) {
    println!("Available commands: help, reboot");
}

fn cmd_reboot(
    _menu: &MyMenuType,
    _item: &MyItemType,
    _args: &[&str], // 修改点
    _interface: &mut ShellInterface, // 新增点
    _context: &mut (),
) {
    println!("Rebooting...");
    unsafe {
        esp_idf_svc::sys::esp_restart();
    }
}

pub const ROOT_MENU: Menu<ShellInterface, ()> = Menu {
    label: "esp32",
    items: &[
        &Item {
            item_type: ItemType::Callback {
                function: cmd_help,
                parameters: &[],
            },
            command: "help",
            help: Some("Show help"),
        },
        &Item {
            item_type: ItemType::Callback {
                function: cmd_reboot,
                parameters: &[],
            },
            command: "reboot",
            help: Some("Restart the device"),
        },
    ],
    entry: None,
    exit: None,
};
