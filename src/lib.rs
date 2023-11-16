pub fn init_log() {
    tracing_subscriber::fmt()
        .with_file(false)
        .with_target(false)
        .with_line_number(false)
        .with_level(true)
        .with_max_level(tracing::Level::INFO)
        .init();
}
