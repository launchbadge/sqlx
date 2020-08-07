use crate::io::{put_length_prefixed, put_portal_name, put_statement_name};
use crate::{PgTypeInfo, Postgres};
use sqlx_core::{error::Error, io::Encode, to_value::ToValue, type_info::TypeInfo};

pub(crate) struct Bind<'a> {
    /// The ID of the destination portal (`None` selects the unnamed portal).
    pub(crate) portal: Option<u32>,

    /// The id of the source prepared statement.
    pub(crate) statement: Option<u32>,

    /// The parameter format codes. Each must presently be zero (text) or one (binary).
    ///
    /// There can be zero to indicate that there are no parameters or that the parameters all use the
    /// default format (text); or one, in which case the specified format code is applied to all
    /// parameters; or it can equal the actual number of parameters.
    pub(crate) formats: &'a [i16],

    /// The type of each parameter.
    pub(crate) parameters: &'a [PgTypeInfo],

    /// The value of each parameter.
    pub(crate) arguments: &'a [&'a dyn ToValue<Postgres>],

    /// The result-column format codes. Each must presently be zero (text) or one (binary).
    ///
    /// There can be zero to indicate that there are no result columns or that the
    /// result columns should all use the default format (text); or one, in which
    /// case the specified format code is applied to all result columns (if any);
    /// or it can equal the actual number of result columns of the query.
    pub(crate) result_formats: &'a [i16],
}

impl Encode<'_> for Bind<'_> {
    fn encode_with(&self, buf: &mut Vec<u8>, _: ()) -> Result<(), Error> {
        buf.push(b'B');

        put_length_prefixed(buf, true, |buf| {
            put_portal_name(buf, self.portal);

            put_statement_name(buf, self.statement);

            buf.extend(&(self.formats.len() as i16).to_be_bytes());

            for &format in self.formats {
                buf.extend(&format.to_be_bytes());
            }

            if self.arguments.len() >= (i16::MAX as usize) {
                return Err(Error::Query("too many arguments to transmit".into()));
            }

            buf.extend(&(self.arguments.len() as i16).to_be_bytes());

            if self.parameters.len() != self.arguments.len() {
                return Err(Error::Query(
                    format!(
                        "expected {} arguments but received {}",
                        self.parameters.len(),
                        self.arguments.len(),
                    )
                    .into(),
                ));
            }

            for (index, (argument, parameter)) in
                self.arguments.iter().zip(self.parameters).enumerate()
            {
                if !argument.accepts(parameter) {
                    return Err(Error::ToArgument {
                        index,
                        source: format!(
                            "mismatched types: Rust type `{}` is not compatible with SQL type `{}`",
                            argument.__type_name(),
                            parameter.name()
                        )
                        .into(),
                    });
                }

                put_length_prefixed(buf, false, |buf| {
                    argument
                        .to_value(parameter, buf)
                        .map_err(|source| Error::ToArgument { index, source })
                })?;
            }

            buf.extend(&(self.result_formats.len() as i16).to_be_bytes());

            for &format in self.result_formats {
                buf.extend(&format.to_be_bytes());
            }

            Ok(())
        })?;

        println!("bind: {:?}", buf);

        Ok(())
    }
}
