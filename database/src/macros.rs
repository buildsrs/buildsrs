macro_rules! statements {
    (
        $($(#[$meta:meta])* fn $procedure:ident ( $( $arg:ident: $type:ty ),* ) $body:tt)*
        $(let $name:ident = $statement:expr;)*
    ) => {
        statements!(@declare, $($procedure ,)* $($name ,)*);
        statements!(@prepare,
            $($procedure => (
                statements!(@procedure, $procedure, statements!(@count, $($arg,)*), $body)
            ),)*
            $($name => $statement,)*
        );
        impl<T: GenericClient> Database<T> {
            $(statements!(@implement, $(#[$meta])* $procedure, $($arg: $type,)*);)*
        }
    };
    (@procedure, $name:ident, $args:expr, ;) => {
        &crate::util::sql_call(stringify!($procedure), $args)
    };
    (@procedure, $name:ident, $args:expr, $body:tt) => {
        $body
    };
    (@declare, $($name:ident ,)*) => {
        #[derive(Clone)]
        /// Prepared statements
        ///
        /// Where possible, all interactions with the database are performed using prepared
        /// statements. This allows for catching issues with statements early (during preparation
        /// time) and improves performance. This type contains all of the prepared statements.
        pub struct Statements {
            $($name: Statement,)*
        }

        impl std::fmt::Debug for Statements {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.debug_struct("Statements").finish()
            }
        }
    };
    (@prepare, $($name:ident => $statement:expr,)*) => {
        impl Statements {
            /// Prepare statements.
            pub async fn prepare(client: &Client) -> Result<Self, Error> {
                Ok(Self {
                    $($name: client.prepare($statement).await?,)*
                })
            }
        }
    };
    (@count,) => {
        0
    };
    (@count, $arg:ident, $($args:ident ,)*) => {
        1 + statements!(@count, $($args,)*)
    };
    (@implement, $(#[$meta:meta])* $procedure:ident, $($arg:ident: $type:ty,)*) => {
        $(#[$meta])*
        pub async fn $procedure(&self, $($arg: $type),*) -> Result<(), Error> {
            self.connection
                .execute(
                    &self.statements.$procedure,
                    &[$(&$arg),*],
                )
                .await?;
            Ok(())
        }
    };
}
