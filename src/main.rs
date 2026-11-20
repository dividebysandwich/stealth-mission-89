// mimalloc returns what it frees to the system (glibc kept the memory the menu scenes free on
// worker threads, gigabytes after a few minutes). Not with `dev`: there std is a shared library
// whose allocations would be freed by mimalloc (src/memory.rs trims glibc instead).
#[cfg(not(any(target_arch = "wasm32", target_os = "android", feature = "dev")))]
#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;
#[cfg(all(not(any(target_arch = "wasm32", target_os = "android")), feature = "dev"))]
use mimalloc as _;

fn main() {
    sm89::run()
}
