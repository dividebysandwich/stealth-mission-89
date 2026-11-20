//! Development builds (`dev`, Bevy and std as shared libraries) can't use mimalloc
//! (src/main.rs): on Linux glibc's arenas keep what the worker threads free (scenery layouts,
//! meshes; gigabytes after a few minutes of menu scenes), so every `TRIM_EVERY` seconds a worker
//! hands the free pages back (`malloc_trim`, ~10-20 ms).

use bevy::prelude::*;

const TRIM_EVERY: f32 = 30.0;

pub struct MemoryPlugin;

impl Plugin for MemoryPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Last, trim);
    }
}

fn trim(time: Res<Time<Real>>, mut since: Local<f32>) {
    *since += time.delta_secs();
    if *since < TRIM_EVERY {
        return;
    }
    *since = 0.0;
    bevy::tasks::AsyncComputeTaskPool::get()
        .spawn(async {
            // SAFETY: only hands free memory back to the system.
            unsafe { libc::malloc_trim(0) };
        })
        .detach();
}
