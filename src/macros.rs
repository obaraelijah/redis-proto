#[macro_export]
macro_rules! make_reader {
    ($id:ident, $name:ident) => {
        macro_rules! $name {
            ($state:expr) => {
                &$state.$id
            };
            ($state:expr, $key:expr) => {
                $state.$id.get($key)
            };
        }
    };
}

#[macro_export]
macro_rules! op_variants {
    ($name:ident, $($variant_name:ident($($arg:ty),*)),*) => {
        lazy_static! {
            pub static ref OP_VARIANTS: Vec<String> = {
                let mut v = Vec::new();
                v.push(format!("{}", stringify!($name)));
                $(
                    v.push(format!("{}", stringify!($variant_name($($arg),*))));
                )*
                v
            };
        }
        crate::as_item! {
            #[derive(Debug, Clone)]
            pub enum $name { $($variant_name($($arg),*),)* }
        }
    };
}