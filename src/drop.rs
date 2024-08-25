pub fn instance(instance: &ash::Instance) {
    unsafe { instance.destroy_instance(None); }
}
