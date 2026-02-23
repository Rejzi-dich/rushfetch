use rushfetch::{SysData, Renderer, config::load_config};

fn main() {
    let config   = load_config();
    let data     = SysData::collect();
    let renderer = Renderer::new(&config, &data);

    renderer.render();
}
