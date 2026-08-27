pub trait DestroyWithInstance {
    fn destroy(&mut self, instance: &ash::Instance);
}

pub trait DestroyWithDevice {
    fn destroy(&mut self, device: &ash::Device);
}
