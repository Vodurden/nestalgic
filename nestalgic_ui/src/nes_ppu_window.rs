use imgui::Ui;
use nestalgic::Nestalgic;

/// Debug Window to inspect the PPU state.
pub struct NesPpuWindow {
    pub open: bool
}

impl NesPpuWindow {
    pub fn render(
        &mut self,
        ui: &Ui,
        nestalgic: &Nestalgic,
    ) {
        if !self.open { return; }

        let window = imgui::Window::new("NES PPU");

        window
            .opened(&mut self.open)
            .build(&ui, || {
                ui.text(format!("ADDR: {:016b}", nestalgic.ppu.addr));
                ui.separator();
                ui.text(format!("PPUCTRL: {:08b}", nestalgic.ppu.ppuctrl.0));
                ui.text(format!("PPUMASK: {:08b}", u8::from(nestalgic.ppu.ppumask)));
                ui.text(format!("PPUSTAT: {:08b}", u8::from(nestalgic.ppu.ppustatus)));
                ui.separator();
                ui.text(format!("OAMADDR: {:08b}", nestalgic.ppu.oam_addr));
            });
    }
}

impl Default for NesPpuWindow {
    fn default() -> Self {
        Self { open: false }
    }
}
