//! iced_spike: interactive minimal Iced + shared-wgpu-device probe.
//! Run: `cargo run --release --bin iced_spike` (Plan 829 T-08, V-06).

fn main() -> iced::Result {
    agp::host::iced_spike::set_mode(agp::host::iced_spike::SpikeMode::Live);
    agp::host::iced_spike::run()
}
