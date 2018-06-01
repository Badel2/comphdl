use stdweb::js_export;

#[js_export]
pub fn run_js_gui() {
    console!(log, "Hello, World!");
}
