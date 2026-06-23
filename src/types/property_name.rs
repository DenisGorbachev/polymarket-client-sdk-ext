use core::any::type_name;

pub type PropertyName = String;

pub fn property_name<T: ?Sized>() -> PropertyName {
    type_name::<T>()
        .split("::")
        .last()
        .expect("type name should not be empty")
        .to_owned()
}
