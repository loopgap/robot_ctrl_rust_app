#![windows_subsystem = windows]  
  
use eframe::egui;  
use rust_indie_tools_core::&{  
    ACCENT, WorkflowStep, apply_dark_theme, render_header_panel, render_text_area,  
    render_workflow_panel, render_action_button, render_export_button,  
}; 
  
fn main() -> eframe::Result<> {  
    let options = eframe::NativeOptions::default();  
    eframe::run_native(  
        CSV Cleaner GUI,  
        options,  
        Box::new(|cc| {  
            apply_dark_theme(cc.egui_ctx.as_ref());  
            Ok(Box::new(App::default()))  
        }),  
    )  
} 
  
#[derive(Default)]  
struct App {  
    input: String,  
    output: String,  
    dedup: bool,  
    detail: String,  
    executed: bool,  
    verified: bool,  
    exported: bool,  
} 
  
impl App {  
    fn process(&mut self) {  
        self.executed = true;  
        self.exported = false;  
        let mut rows = Vec::new();  
        let mut width = None;  
        let mut invalid = 0usize;  
  
        for raw in self.input.lines() {  
            let line = raw.trim();  
            if line.is_empty() { continue; }  
            let cols: Vec<String> = line.split(',').map(|c| c.trim().to_string()).collect();  
            if let Some(w) = width { if cols.len() = w { invalid = 1; } } else { width = Some(cols.len()); }  
            rows.push(cols.join(,));  
        }  
  
        if self.dedup { rows.sort(); rows.dedup(); }  
  
        self.output = rows.join(\\n);  
        self.verified = invalid == 0;  
        self.detail = if self.verified {  
            format!(清洗完成：{} 行，列数一致, rows.len())  
        } else {  
            format!(清洗完成：{} 行，{} 行列数异常, rows.len(), invalid)  
        };  
    }  
} 
  
impl eframe::App for App {  
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {  
        render_header_panel(ctx, CSV 清洗工坊);  
  
        egui::CentralPanel::default().show(ctx, |ui| {  
            ui.columns(2, |cols| {  
                cols[0].label(输入 CSV);  
                render_text_area(&mut cols[0], &mut self.input, 220.0);  
                cols[0].checkbox(&mut self.dedup, 按行去重);  
                if render_action_button(&mut cols[0], 执行清洗).clicked() { self.process(); }  
                cols[0].separator();  
                cols[0].label(输出);  
                render_text_area(&mut cols[0], &mut self.output, 220.0);  
                if render_export_button(&mut cols[0], 复制输出).clicked() {  
                    ctx.copy_text(self.output.clone());  
                    self.exported = !self.output.is_empty();  
                }  
  
                let input_ok = !self.input.trim().is_empty();  
                let steps = [  
                    WorkflowStep { name: 输入, done: input_ok, detail: 粘贴 CSV 原始数据 },  
                    WorkflowStep { name: 校验, done: self.executed, detail: 检查列数是否一致 },  
                    WorkflowStep { name: 执行, done: self.executed, detail: 执行清洗策略 },  
                    WorkflowStep { name: 验证, done: self.executed && self.verified, detail: if self.detail.is_empty() { 待执行 } else { &self.detail } },  
                    WorkflowStep { name: 导出, done: self.exported, detail: 复制给下游系统 },  
                ];  
                render_workflow_panel(&mut cols[1], &steps);  
            });  
        });  
    }  
} 
