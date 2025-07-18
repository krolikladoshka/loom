macro_rules! require {
    ($cond:expr $(,)?) => {
        assert!($cond)
    };
    ($cond:expr, $($arg:tt)+) => {
        assert!($cond, $($arg)+)
    };
}

#[macro_export]
macro_rules! dev_assert {
    ($cond:expr $(,)?) => {
        #[cfg(feature = "enable-asserts")]
        {
            assert!($cond);
        }
    };
    ($cond:expr, $($arg:tt)+) => {
        #[cfg(feature = "enable-asserts")]
        {
            assert!($cond, $($arg)+);
        }
    };
}

#[macro_export]
macro_rules! dev_assert_eq {
    ($left:expr, $right:expr $(,)?) => {
        #[cfg(feature = "enable-asserts")]
        {
            assert_eq!($left, $right);
        }
    };
    ($left:expr, $right:expr, $($arg:tt)+) => {
        #[cfg(feature = "enable-asserts")]
        {
            assert_eq!($left, $right, $($arg)+);
        }
    };
}

#[macro_export]
macro_rules! dev_assert_ne {
    ($left:expr, $right:expr $(,)?) => {
        #[cfg(feature = "enable-asserts")]
        {
            assert_ne!($left, $right);
        }
    };
    ($left:expr, $right:expr, $($arg:tt)+) => {
        #[cfg(feature = "enable-asserts")]
        {
            assert_ne!($left, $right, $($arg)+);
        }
    };
}