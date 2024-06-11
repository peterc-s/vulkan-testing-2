pub mod constants;

pub mod app_data {
    use vulkanalia::vk::DebugUtilsMessengerEXT;

    #[derive(Clone, Debug)]
    pub struct DebugMessenger {
        pub messenger: DebugUtilsMessengerEXT,
    }
}

pub mod app {
    /*
     * imports
     */

    use super::constants::*;
    use super::app_data::*;

    use anyhow::{anyhow, Result};
    use log::*;
    
    use vulkanalia::vk::ExtDebugUtilsExtension;
    use winit::window::Window;

    use vulkanalia::{
        loader::{LibloadingLoader, LIBRARY},
        window as vk_window,
        prelude::v1_0::*,
    };

    use std::{
        collections::HashSet,
        ffi::CStr,
        os::raw::c_void,
    };

    /*
     * the vulkan app
     */

    #[derive(Clone, Debug)]
    pub struct App {
        pub entry: Entry,
        pub instance: Instance,
        pub debug_messenger: Option<DebugMessenger>,
    }

    impl App {
        pub unsafe fn create(window: &Window) -> Result<Self> {
            // create loader, entry, and instance
            let loader = LibloadingLoader::new(LIBRARY)?;
            let entry = Entry::new(loader).map_err(|b| anyhow!("{}", b))?;
            let instance = create_instance(window, &entry)?;

            // create the debug messenger for the validation layer
            let debug_messenger = if VALIDATION_ENABLED {
                Some(create_debug_messenger(&instance)?)
            } else {
                None
            };

            Ok(Self {
                entry,
                instance,
                debug_messenger,
            })
        }

        pub unsafe fn render(&mut self, window: &Window) -> Result<()> {
            Ok(())
        }

        pub unsafe fn destroy(&mut self) {
            // destroy the debug messener if it exists
            match &self.debug_messenger {
                Some(debug_messenger) => self.instance.destroy_debug_utils_messenger_ext(debug_messenger.messenger, None),
                _ => {}
            };

            self.instance.destroy_instance(None);
        }
    }

    /*
     * creation functions
     */

    unsafe fn create_instance(window: &Window, entry: &Entry) -> Result<Instance> {
        // create application info struct
        let application_info = vk::ApplicationInfo::builder()
            .application_name(b"Vulkan Testing\0")
            .application_version(vk::make_version(1, 0, 0))
            .engine_name(b"No Engine\0")
            .engine_version(vk::make_version(1, 0, 0))
            .api_version(vk::make_version(1, 0, 0));

        // get available layer names in a hashset
        let available_layers = entry
            .enumerate_instance_layer_properties()?
            .iter()
            .map(|l| l.layer_name)
            .collect::<HashSet<_>>();

        // check if validation enabled and supported
        if VALIDATION_ENABLED && !available_layers.contains(&VALIDATION_LAYER) {
            return Err(anyhow!("Validation layer requested but not supported."));
        }

        // add validation layer if enabled
        let layers = if VALIDATION_ENABLED {
            vec![VALIDATION_LAYER.as_ptr()]
        } else {
            Vec::new()
        };

        // get required extensions
        let mut extensions = vk_window::get_required_instance_extensions(window)
            .iter()
            .map(|e| e.as_ptr())
            .collect::<Vec<_>>();

        // get flags if target is macos
        let flags = if cfg!(target_os = "macos") &&
            entry.version()? >= PORTABILITY_MACOS_VERSION {
            info!("Enabling extensions for MacOS portability.");
            extensions.push(vk::KHR_GET_PHYSICAL_DEVICE_PROPERTIES2_EXTENSION.name.as_ptr());
            extensions.push(vk::KHR_PORTABILITY_ENUMERATION_EXTENSION.name.as_ptr());
            vk::InstanceCreateFlags::ENUMERATE_PORTABILITY_KHR
        } else {
            vk::InstanceCreateFlags::empty()
        };

        let info = vk::InstanceCreateInfo::builder()
            .application_info(&application_info)
            .enabled_layer_names(&layers)
            .enabled_extension_names(&extensions)
            .flags(flags);

        Ok(entry.create_instance(&info, None)?)
    }

    unsafe fn create_debug_messenger(instance: &Instance) -> Result<DebugMessenger> {
        assert!(VALIDATION_ENABLED);

        let debug_info = vk::DebugUtilsMessengerCreateInfoEXT::builder()
            .message_severity(vk::DebugUtilsMessageSeverityFlagsEXT::all())
            .message_type(vk::DebugUtilsMessageTypeFlagsEXT::all())
            .user_callback(Some(debug_callback));

        let messenger = instance.create_debug_utils_messenger_ext(&debug_info, None)?;

        Ok(DebugMessenger {
            messenger,
        })
    }

    // debug callback for validation layer
    extern "system" fn debug_callback(
        severity: vk::DebugUtilsMessageSeverityFlagsEXT,
        type_: vk::DebugUtilsMessageTypeFlagsEXT,
        data: *const vk::DebugUtilsMessengerCallbackDataEXT,
        _: *mut c_void,
    ) -> vk::Bool32 {
        let data = unsafe { *data };
        let message = unsafe { CStr::from_ptr(data.message) }.to_string_lossy();

        if severity >= vk::DebugUtilsMessageSeverityFlagsEXT::ERROR {
            error!("({:?}) {}", type_, message);
        } else if severity >= vk::DebugUtilsMessageSeverityFlagsEXT::WARNING {
            warn!("({:?}) {}", type_, message);
        } else if severity >= vk::DebugUtilsMessageSeverityFlagsEXT::INFO {
            debug!("({:?}) {}", type_, message);
        } else {
            trace!("({:?}) {}", type_, message);
        }

        vk::FALSE
    }
}
