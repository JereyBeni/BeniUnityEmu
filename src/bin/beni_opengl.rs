//! BeniOpenGL — Stage C1 partial OpenGL runtime window.
//! Does **not** load or execute Unity guest games.

fn main() {
    println!("BeniUnityEmu OpenGL Runtime [PARTIAL]");
    println!("Unity guest execution: NOT IMPLEMENTED");
    println!();

    match beni_unity_emu::graphics::run_opengl_runtime_from_config() {
        Ok(()) => println!("[OpenGL] Window closed cleanly."),
        Err(e) => {
            eprintln!("[OpenGL] FATAL: {}", e);
            eprintln!("No software/fake renderer fallback was used.");
            std::process::exit(1);
        }
    }
}
