use kovi::build_bot;

fn main() {
    build_bot!(
        kovi_plugin_cmd,
        kovi_plugin_like
    ).run();
}
