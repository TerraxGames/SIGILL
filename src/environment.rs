#[macro_export]
macro_rules! wrong_side {
    ( $side:expr ) => {
        panic!("Method called on incorrect side {}!", $side);
    };
}

#[macro_export]
macro_rules! sided {
    ( $target_side:expr, $side:expr, $code:block ) => {
        if $target_side == $side {
            $code
        } else {
            $crate::wrong_side!($side);
        }
    };
}

#[macro_export]
macro_rules! client_only {
    ( $side:expr, $code:block ) => {
        $crate::sided!($crate::environment::Side::Client, $side, $code)
    };
}

#[macro_export]
macro_rules! dedicated_server_only {
    ( $side:expr, $code:expr ) => {
        $crate::sided!($crate::environment::Side::DedicatedServer, $side, $code)
    };
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Side {
    Client,
    DedicatedServer,
}

impl core::fmt::Display for Side {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Client { .. } => f.write_str("client"),
            Self::DedicatedServer => f.write_str("dedicated_server"),
        }
    }
}
