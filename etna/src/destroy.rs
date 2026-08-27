use crate::{device::Device, instance::Instance};

pub trait DestroyWithInstance {
    fn destroy(&mut self, instance: &Instance);
}

pub trait DestroyWithDevice {
    fn destroy_vk(&mut self, device: &ash::Device) {}
    fn destroy_ext(&mut self, device: &Device) {}
}
