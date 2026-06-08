mod bridge;

use godot::prelude::*;

struct TinyGroveExtension;

#[gdextension]
unsafe impl ExtensionLibrary for TinyGroveExtension {}
