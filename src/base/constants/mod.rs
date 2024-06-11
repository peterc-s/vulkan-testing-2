use vulkanalia::{
    Version,
    vk::ExtensionName,
};

pub const PORTABILITY_MACOS_VERSION: Version = Version::new(1, 3, 216);
pub const VALIDATION_ENABLED: bool = cfg!(debug_assertions);
pub const VALIDATION_LAYER: ExtensionName = ExtensionName::from_bytes(b"VK_LAYER_KHRONOS_validation");
