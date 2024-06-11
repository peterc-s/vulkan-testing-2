pub mod app {
    use anyhow::{anyhow, Result};
    use log::*;
    
    use winit::window::Window;

    use vulkanalia::{
        loader::{LibloadingLoader, LIBRARY},
        window as vk_window,
        prelude::v1_0::*,
    };
    
    #[derive(Clone, Debug)]
    pub struct App {
        pub entry: Entry,
        pub instance: Instance,
    }

    impl App {
        pub unsafe fn create(window: &Window) -> Result<Self> {
            // create loader, entry, and instance
            let loader = LibloadingLoader::new(LIBRARY)?;
            let entry = Entry::new(loader).map_err(|b| anyhow!("{}", b))?;
            let instance = create_instance(window, &entry)?;


            Ok(Self {
                entry,
                instance,
            })
        }

        pub unsafe fn render(&mut self, window: &Window) -> Result<()> {
            Ok(())
        }

        pub unsafe fn destroy(&mut self) {
            self.instance.destroy_instance(None);
        }
    }

    unsafe fn create_instance(window: &Window, entry: &Entry) -> Result<Instance> {
        let application_info = vk::ApplicationInfo::builder()
            .application_name(b"Vulkan Testing\0")
            .application_version(vk::make_version(1, 0, 0))
            .engine_name(b"No Engine\0")
            .engine_version(vk::make_version(1, 0, 0))
            .api_version(vk::make_version(1, 0, 0));

        let extensions = vk_window::get_required_instance_extensions(window)
            .iter()
            .map(|e| e.as_ptr())
            .collect::<Vec<_>>();

        let info = vk::InstanceCreateInfo::builder()
            .application_info(&application_info)
            .enabled_extension_names(&extensions);

        Ok(entry.create_instance(&info, None)?)
    }
}
