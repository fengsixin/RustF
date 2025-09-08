use eframe::{egui, App, Frame};
use crate::state::MyApp;

impl App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut Frame) {
        self.check_for_conversion_result();
        self.check_for_import_result();

        let mut request_repaint = false;

        if ctx.input(|i| i.key_pressed(egui::Key::B) && i.modifiers.ctrl) {
            ctx.input_mut(|i| i.consume_key(egui::Modifiers::CTRL, egui::Key::B));
            self.apply_formatting_to_selection(ctx, "**", "**");
            request_repaint = true;
        }
        
        if ctx.input(|i| i.key_pressed(egui::Key::I) && i.modifiers.ctrl) {
            ctx.input_mut(|i| i.consume_key(egui::Modifiers::CTRL, egui::Key::I));
            self.apply_formatting_to_selection(ctx, "*", "*");
            request_repaint = true;
        }
        
        if ctx.input(|i| i.key_pressed(egui::Key::U) && i.modifiers.ctrl) {
            ctx.input_mut(|i| i.consume_key(egui::Modifiers::CTRL, egui::Key::U));
            self.apply_formatting_to_selection(ctx, "[", "]{.underline}");
            request_repaint = true;
        }

        if ctx.input(|i| i.key_pressed(egui::Key::H) && i.modifiers.ctrl) {
            ctx.input_mut(|i| i.consume_key(egui::Modifiers::CTRL, egui::Key::H));
            self.apply_formatting_to_selection(ctx, "{{", "}}");
            request_repaint = true;
        }

        if request_repaint {
            ctx.request_repaint();
        }

        if ctx.input(|i| i.key_pressed(egui::Key::S) && i.modifiers.ctrl && i.modifiers.shift) {
            ctx.input_mut(|i| i.consume_key(egui::Modifiers::CTRL | egui::Modifiers::SHIFT, egui::Key::S));
            
            self.style_palette_open = !self.style_palette_open;

            if self.style_palette_open {
                self.palette_search_text.clear();
                self.palette_selected_index = 0;
                self.update_filtered_styles(); 
            }
        }

        self.show_menu_bar(ctx);
        
        if self.about_window_open {
            self.show_about_window(ctx);
        }

        if self.assignment_window_open {
            self.show_assignment_window(ctx);
        }

        if self.style_palette_open {
            self.show_style_palette(ctx);
        }
        
        self.show_panels(ctx);
    }
}
