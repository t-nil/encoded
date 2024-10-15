use color_eyre::Result;

pub trait Param: Send + Sync {
    fn key() -> String
    where
        Self: Sized; // because of case conversion
    fn val(&self) -> String;
}

macro_rules! init_param_map {
    () => {
        static PARAMS: LazyLock<
            std::sync::Mutex<
                std::collections::HashMap<
                    String,
                    std::sync::Mutex<Box<dyn Fn(String) -> Result<Box<dyn Param>> + Send + Sync>>,
                >,
            >,
        > = std::sync::LazyLock::new(|| std::sync::Mutex::new(Default::default()));
    };
}

macro_rules! param {
    ([$($trait:ty),+],$gen_type:ident, $internal_name:ident : $internal_type:ty, $default:expr, $func:block) =>
    {
        #[derive(Clone, PartialEq, Eq, Hash, Debug, proc_macros::ConditionalCopy, ::serde::Serialize, ::serde::Deserialize)]
        pub struct $gen_type {
            $internal_name: $internal_type,
        }

        impl $crate::video::encoder::params::Param for $gen_type {
            fn key() -> String {
                use ::convert_case::{Case, Casing};
                std::any::type_name::<$gen_type>().to_case(Case::Snake)
            }
            fn val(&self) -> String {
                self.$internal_name.to_string()
            }
        }

        impl TryFrom<$internal_type> for $gen_type {
            type Error = color_eyre::Report;

            fn try_from(
                $internal_name: $internal_type,
            ) -> std::prelude::v1::Result<Self, Self::Error> {
                $func
                Ok(Self { $internal_name })
            }
        }

        impl From<$gen_type> for $internal_type {
            fn from($internal_name: $gen_type) -> $internal_type {
                $internal_name.$internal_name
            }
        }

        impl AsRef<$internal_type> for $gen_type {
            fn as_ref<'a>(&'a self) -> &'a $internal_type {
                &self.$internal_name
            }
        }

        impl Default for $gen_type {
            fn default() -> Self {
                Self { $internal_name: $default }
            }
        }

        $(
            impl $trait for $gen_type{}
        )+
    };
    ($gen_type:ident, $internal_name:ident : $internal_type:ty, $default:expr, $func:block) => {
        param!([Param], $gen_type, $internal_name : $internal_type, $default, $func);
    };
}
pub(crate) use param;

#[derive(Clone, Debug, Default, PartialEq)]
pub enum EncodeSettings {
    #[default]
    X265,
    Av1Svt(/*svt_av1::Settings*/),
    Av1Aom,
    Av1Rav1e,
}

pub trait EncoderSettings {
    fn effective_path(&self) -> String;
}

impl EncodeSettings {
    fn effective_path(&self) -> Result<String> {
        todo!()
    }
}
pub mod sample {
    pub struct Settings {}
}

mod tests {
    use color_eyre::Result;
    fn test_effective_path() -> Result<()> {
        todo!()
    }
}
