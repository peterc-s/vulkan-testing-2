pub mod constants;

pub mod app {
    /*
     * imports
     */

    use super::constants::*;
    use data::QueueFamilyIndices;

    use thiserror::Error;
    use anyhow::{anyhow, Result};
    use log::*;
    
    use winit::window::Window;

    use vulkanalia::{
        loader::{LibloadingLoader, LIBRARY},
        window as vk_window,
        prelude::v1_0::*,
        vk::{DebugUtilsMessengerEXT, ExtDebugUtilsExtension},
        Instance,
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
        pub debug_messenger: Option<DebugUtilsMessengerEXT>,
        pub phys_device: vk::PhysicalDevice,
    }

    impl App {
        pub unsafe fn create(window: &Window) -> Result<Self> {
            // create loader, entry, and instance
            let loader = LibloadingLoader::new(LIBRARY)?;
            let entry = Entry::new(loader).map_err(|b| anyhow!("{}", b))?;
            let (instance, debug_messenger) = create_instance(window, &entry)?;

            let phys_device = choose_physical_device(&instance)?;

            Ok(Self {
                entry,
                instance,
                debug_messenger,
                phys_device,
            })
        }

        pub unsafe fn render(&mut self, window: &Window) -> Result<()> {
            Ok(())
        }

        pub unsafe fn destroy(&mut self) {
            // destroy the debug messener if it exists
            match self.debug_messenger {
                Some(messenger) => self.instance.destroy_debug_utils_messenger_ext(messenger, None),
                _ => {}
            };

            self.instance.destroy_instance(None);
        }
    }

    /*
     * creation functions
     */

    unsafe fn create_instance(window: &Window, entry: &Entry) -> Result<(Instance, Option<DebugUtilsMessengerEXT>)> {
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

        if VALIDATION_ENABLED {
            extensions.push(vk::EXT_DEBUG_UTILS_EXTENSION.name.as_ptr());
        }

        let mut info = vk::InstanceCreateInfo::builder()
            .application_info(&application_info)
            .enabled_layer_names(&layers)
            .enabled_extension_names(&extensions)
            .flags(flags);

        // set up validation for create instance call if enabled
        let mut debug_messenger: Option<DebugUtilsMessengerEXT> = None;

        let mut debug_info = vk::DebugUtilsMessengerCreateInfoEXT::builder()
                .message_severity(vk::DebugUtilsMessageSeverityFlagsEXT::all())
                .message_type(vk::DebugUtilsMessageTypeFlagsEXT::all())
                .user_callback(Some(debug_callback));

        if VALIDATION_ENABLED {
            info = info.push_next(&mut debug_info);
        }

        let instance = entry.create_instance(&info, None)?;

        // create debug messenger if validation is enabled
        if VALIDATION_ENABLED {
            debug_messenger = Some(instance.create_debug_utils_messenger_ext(&debug_info, None)?);
        }
        
        Ok((instance, debug_messenger))
    }

    // used for GPU suitability
    #[derive(Debug, Error)]
    #[error("Missing {0}.")]
    pub struct SuitabilityError(pub &'static str);

    unsafe fn choose_physical_device(instance: &Instance) -> Result<vk::PhysicalDevice> {
        for phys_device in instance.enumerate_physical_devices()? {
            let properties = instance.get_physical_device_properties(phys_device);

            if let Err(error) = check_physical_device(instance, phys_device) {
                warn!("Skipping physical device ({}): {}", properties.device_name, error)
            } else {
                info!("Selected physical device ({})", properties.device_name);
                return Ok(phys_device);
            }
        }
        
        Err(anyhow!("Failed to find suitable physical device."))
    }

    unsafe fn check_physical_device(
        instance: &Instance,
        phys_device: vk::PhysicalDevice
    ) -> Result<()> {
        QueueFamilyIndices::get(instance, phys_device)?;
        Ok(())
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
    
    pub mod data {
        use super::SuitabilityError;
        use vulkanalia::{Instance, vk, prelude::v1_0::*};
        use anyhow::{Result, anyhow};

        #[derive(Copy, Clone, Debug)]
        pub struct QueueFamilyIndices {
            pub graphics: u32,
        }

        impl QueueFamilyIndices {
            pub unsafe fn get(
                instance: &Instance,
                phys_device: vk::PhysicalDevice,
            ) -> Result<Self> {
                let properties = instance
                    .get_physical_device_queue_family_properties(phys_device);

                let graphics = properties
                    .iter()
                    .position(|p| p.queue_flags.contains(vk::QueueFlags::GRAPHICS))
                    .map(|i| i as u32);

                if let Some(graphics) = graphics {
                    Ok(Self{ graphics })
                } else {
                    Err(anyhow!(SuitabilityError("Missing required queue families.")))
                }
            }
        }


    }
}
