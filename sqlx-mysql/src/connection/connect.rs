//! Implements the connection phase.
//!
//! The connection phase (establish) performs these tasks:
//!
//! -   exchange the capabilities of client and server
//! -   setup SSL communication channel if requested
//! -   authenticate the client against the server
//!
//! The server may immediately send an ERR packet and finish the handshake
//! or send a `Handshake`.
//!
//! https://dev.mysql.com/doc/internals/en/connection-phase.html
//!
use sqlx_core::Result;

use crate::protocol::{Auth, AuthResponse, Handshake, HandshakeResponse};
use crate::{MySqlConnectOptions, MySqlConnection};

macro_rules! connect {
    (@blocking @tcp $options:ident) => {
        Rt::connect_tcp($options.get_host(), $options.get_port())?;
    };

    (@tcp $options:ident) => {
        Rt::connect_tcp_async($options.get_host(), $options.get_port()).await?;
    };

    (@blocking @packet $self:ident) => {
        $self.read_packet()?;
    };

    (@packet $self:ident) => {
        $self.read_packet_async().await?;
    };

    ($(@$blocking:ident)? $options:ident) => {{
        // open a network stream to the database server
        let stream = connect!($(@$blocking)? @tcp $options);

        // construct a <MySqlConnection> around the network stream
        // wraps the stream in a <BufStream> to buffer read and write
        let mut self_ = Self::new(stream);

        // immediately the server should emit a <Handshake> packet
        let handshake: Handshake = connect!($(@$blocking)? @packet self_);

        // & the declared server capabilities with our capabilities to find
        // what rules the client should operate under
        self_.capabilities &= handshake.capabilities;

        // store the connection ID, mainly for debugging
        self_.connection_id = handshake.connection_id;

        // extract the auth plugin and data from the handshake
        // this can get overwritten by an auth switch
        let mut auth_plugin = handshake.auth_plugin;
        let mut auth_plugin_data = handshake.auth_plugin_data;
        let password = $options.get_password().unwrap_or_default();

        // create the initial auth response
        // this may just be a request for an RSA public key
        let initial_auth_response = auth_plugin.invoke(&auth_plugin_data, password);

        // the <HandshakeResponse> contains an initial guess at the correct encoding of
        // the password and some other metadata like "which database", "which user", etc.
        self_.write_packet(&HandshakeResponse {
            auth_plugin_name: auth_plugin.name(),
            auth_response: initial_auth_response,
            charset: 45, // [utf8mb4]
            database: $options.get_database(),
            max_packet_size: 1024,
            username: $options.get_username(),
        })?;

        loop {
            match connect!($(@$blocking)? @packet self_) {
                Auth::Ok(_) => {
                    // successful, simple authentication; good to go
                    break;
                }

                Auth::MoreData(data) => {
                    if let Some(data) = auth_plugin.handle(data, &auth_plugin_data, password)? {
                        // write the response from the plugin
                        self_.write_packet(&AuthResponse { data })?;

                        // let's try again
                        continue;
                    }

                    // all done, the plugin says we check out
                    break;
                }

                Auth::Switch(sw) => {
                    // switch to the new plugin
                    auth_plugin = sw.plugin;
                    auth_plugin_data = sw.plugin_data;

                    // generate an initial response from this plugin
                    let data = auth_plugin.invoke(&auth_plugin_data, password);

                    // write the response from the plugin
                    self_.write_packet(&AuthResponse { data })?;

                    // let's try again
                    continue;
                }
            }
        }

        Ok(self_)
    }};
}

#[cfg(feature = "async")]
impl<Rt> MySqlConnection<Rt>
where
    Rt: sqlx_core::Runtime,
{
    pub(crate) async fn connect_async(options: &MySqlConnectOptions<Rt>) -> Result<Self>
    where
        Rt: sqlx_core::Async,
        for<'s> Rt::TcpStream: sqlx_core::io::Stream<'s, Rt>,
    {
        connect!(options)
    }
}

#[cfg(feature = "blocking")]
impl<Rt> MySqlConnection<Rt>
where
    Rt: sqlx_core::blocking::Runtime,
{
    pub(crate) fn connect(options: &MySqlConnectOptions<Rt>) -> Result<Self>
    where
        for<'s> Rt::TcpStream: sqlx_core::blocking::io::Stream<'s, Rt>,
    {
        connect!(@blocking options)
    }
}

#[cfg(all(test, feature = "async"))]
mod tests {
    use futures_executor::block_on;
    use sqlx_core::{mock::Mock, ConnectOptions};

    use crate::mock::MySqlMockStreamExt;
    use crate::MySqlConnectOptions;

    const SRV_HANDSHAKE_DEFAULT_OLD_AUTH: &[u8] = b"\n5.5.5-10.5.8-MariaDB-1:10.5.8+maria~focal\0)\0\0\04bo+$r4H\0\xfe\xf7-\x02\0\xff\x81\x15\0\0\0\0\0\0\x0f\0\0\0O5X>j}Ur]Y)^\0mysql_old_password\0";
    const SRV_HANDSHAKE_DEFAULT_NATIVE_AUTH: &[u8] = b"\n5.5.5-10.5.8-MariaDB-1:10.5.8+maria~focal\0)\0\0\04bo+$r4H\0\xfe\xf7-\x02\0\xff\x81\x15\0\0\0\0\0\0\x0f\0\0\0O5X>j}Ur]Y)^\0mysql_native_password\0";
    const SRV_HANDSHAKE_DEFAULT_CACHING_SHA2_AUTH: &[u8] = b"\n8.0.22\0\x08\0\0\0TIbl}%U#\0\xff\xff\xff\x02\0\xff\xc7\x15\0\0\0\0\0\0\0\0\0\0\x06\x12\x0e`5\x1b\x12\x0b\x13\x06_\x19\0caching_sha2_password\0";
    const SRV_HANDSHAKE_DEFAULT_SHA256_AUTH: &[u8] = b"\n8.0.22\0\x0e\0\0\0\x1b\x02O\x04hL8D\0\xff\xff\xff\x02\0\xff\xc7\x15\0\0\0\0\0\0\0\0\0\0^*Nh\x19\x1f*)-\x0c\x07v\0sha256_password\0";

    const SRV_PUBLIC_KEY: &[u8] = b"\x01-----BEGIN PUBLIC KEY-----\nMIIBIjANBgkqhkiG9w0BAQEFAAOCAQ8AMIIBCgKCAQEAwnXi3nr9TmN+NF49A3Y7\nUBnAVhApNJy2cmuf/y6vFM9eHFu5T80Ij1qYc6c79oAGA8nNNCFQL+0j5De88cln\nKrlzq/Ab3U+j5SqgNwk//F6Y3iyjV4L7feSDqjpcheFzkjEslbm/yoRwQ78AAU6s\nqA0hcFuh66mcvnotDrvZAGQ8U2EbbZa6oiR3wrgbzifSKq767g65zIrCpoyxzKMH\nAETSDIaMKpFio4dRATKT5ASQtPoIyxSBmjRtc22sqlhEeiejEMsJzd6Bliuait+A\nkTXL6G1Tbam26Dok/L88CnTAWAkLwTA3bjPcS8Zl9gTsJvoiMuwW1UPEVV/aJ11Z\n/wIDAQAB\n-----END PUBLIC KEY-----\n";
    const SRV_AUTH_OK: &[u8] = b"\0\0\0\x02\0\0\0";
    const SRV_AUTH_MORE_CONTINUE: &[u8] = b"\x01\x04";
    const SRV_AUTH_MORE_OK: &[u8] = b"\x01\x03";
    const SRV_SWITCH_CACHING_SHA2_AUTH: &[u8] =
        b"\xfecaching_sha2_password\0\x12}Wz?0-M9sO*S\x03\nP\x1c]pe\0";
    const SRV_SWITCH_NATIVE_AUTH: &[u8] =
        b"\xfemysql_native_password\0\r.89j]CpA3Ov~\x1de\\/\x15,\r\0";

    const RES_HANDSHAKE_NATIVE_AUTH: &[u8] = b"P\0\0\x01\x04\xa3(\x01\0\x04\0\0-\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0root\0\x14P\xaf\xf1\x12,\xe9\xad\xea\x7f\xa0\n\xcd\xa2\xb5<\x17\xa5\xc9J\xd0mysql_native_password\0";
    const RES_HANDSHAKE_EMPTY_AUTH: &[u8] = b"<\0\0\x01\x04\xa3(\x01\0\x04\0\0-\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0root\0\0mysql_native_password\0";
    const RES_HANDSHAKE_CACHING_SHA2_AUTH: &[u8] = b"\\\0\0\x01\x05\xa3(\x01\0\x04\0\0-\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0root\0 \x9d\x85T\x15\xfe\xa9u\x13\x02&\x9dlG\x17\x98\x1b`\x8a\x96\xfcI\x19\x17\xe0(I8\xba\xd7\xfax\xa9caching_sha2_password\0";
    const RES_HANDSHAKE_SHA256_AUTH: &[u8] = b"7\0\0\x01\x05\xa3(\x01\0\x04\0\0-\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0root\0\x01\x01sha256_password\0";

    const RES_ASK_RSA_KEY: &[u8] = b"\x01\0\0\x03\x02";
    const RES_ASK_RSA_KEY_2: &[u8] = b"\x01\0\0\x05\x02";
    const RES_RSA_PASSWORD_SHA256: &[u8] = b"\0\x01\0\x03\xc1*\xf5=\xc3\x86\x95U$=\x9c \x946_Rg\xdc\x9d\xa0M\xf2@\xba\xf7\x8f\rE\xdbrI\xac\x05\xfb\xd1\xaa\r0 '\xf2\xec\xb3Xu\x98\x82\xf2\x8d)\x80\xe7\xdcG\\\xde\x87\x0e\x07\x87f\xach\xbb\x0b\xdf\xe0\xd9\xd1N\x9f_\x17xT\xec\xd5\xff\xd3\xa35\x11PO\xca\xf2\x13?=n\xe7\xd5\xbb\xa0\xd0\xca\xc5\x80\xb0\0\xc0\xe9F\x90f\xa0a\xd1\xdb\xe4(\xed2\xd7@\xb8u\x859U\xd6\xa2\xc3\xa2\xbe\x9a\xeeSy\x92\x95\r\xd3\x14\x90\x80\xb1o#\xa6\x7f\x16\x7f\t-'\xf35\xa02zY\xaeP^e\xf9O\xed\x9d\xb5\x8b\x9d\x0cayA\xff\"-\x80\x8c<\xc4\x11e\xdf\x9c\xe2\x9b)\x8f\xb0\xe9\xe1\xbcj\xf9\xa0U\xe6\x95\x9b\x01 \xba\x7f\"\\\x0cF9\\'\xf2\xfcMD\x1a\xd8\xe3\x11\xdfN\xc4\xd3\x9e\xee\x8d\r\xda\x94\xc4\xafR\xf3\x1e8b\x8d$\x84Nj\x18~\xa7\xf1\x8bb&\x90\xc0\xad\xb1O\xec\xfa\x98h\xf0{.\x07R\n";
    const RES_RSA_PASSWORD_CACHING_SHA2: &[u8] = b"\0\x01\0\x05#7\x8f\xd6\x8dCi9*\xee\x87\xb3\xb1,@\xdf\x94\xa8g\xbf\xed5\xf3\x1e\x9c\xfe\xda\xe8-6\x9c\x1eO\xb6\x80\x81]h\x0b\xd8\x10xx\xeb\x8b\xe9\x8a\x93\xd7\x83\xf7\x9a\xe1\xb94\xfd\xb0\x81\xeb\x0f\xecU:\xf4\x82\x11\xd3\xee\x8e+\x9e_rm\xb4\xbdM\xa0\x90\xff\xc3\x03V*\xa6|\x16\xdd\xea\xd2\x92\xef\xf5E\xb1t\n\xb7\xd9\x8bU\xbd\x94\xb8\x80|S+z\x1bO\x1e\xdf&\xf7(\xf0~\x97\x8b\xee1\xa4\xbb\x9f6\xc4\x88\xbf\x14$\xb2\xc0\xea\x9f\xdd\xfc\x99\xc8\xfe\x178\xf3X\x90\x01\xcc\xa8\x86\x9d\xe9\x98\xbf\xc2\xdc\xe8\xff\x96\xbd^\xf6\r \xb5\xe8\x0euo\xb5(\x80\xffW7\xf0\xdd\xcc\xaa\x9fYl\xef\xb7y\xf7A\xf4\xcf\x1f\xfc\rS\x7f\x13\xa9b\xadd\x1c\xcf\xf5\x98\x0ei\xc3\x0f\x9c\x8eqeTu\x8b\x17\xe7\xd47\xc5\xe9j=\xfc\x82\x04\x96}V.U?\x85\x14J\xe2\xd3.+:\xc5\xe0'm\x9a3\x85\x1e\xf7\xad\xf9J\xcf\xfc\xa7\xc2\x04@";
    const RES_SHA_SCRAMBLE: &[u8] = b" \0\0\x03\xffjg\x06p\x1d\xeawto\xf3\xf6\xa0\x9f7\xa9Z\xb3\xa5\xf9\x0b\x80\x14j8WTb\xf1{f\xf5";
    const RES_NATIVE_SCRAMBLE: &[u8] =
        b"\x14\0\0\x031.Z\x95JON\x81\x9ak\xc7\xba\xe6{L\x0f\xe8\x03N\xef";
    const RES_SWITCH_RSA_PASSWORD_CACHING_SHA2: &[u8] = b"\0\x01\0\x077fS:\x9d3\xec\xe47\xbe\xda\xd8a\x14\x7f\xa8\xa82\x15\xb3\xb8\xa4D\x8f\x8e,,\xc4\x7f\x9ck\x9cI2&\xc2a\xd4\xef\r\x04\xc2\xd1\x89\xb05\xab\xe2YL\xd2hz\xf6y\xb7\xcb\x08\x9a\x1d\xc0A\x7f\x97\xba*\x1e,c\xbcP\xab\xa2\xee\xfa\xcd^=\x1flj\x96\x8fGx\x8e\x9b\xfd\xea\xd05w\xcc\xf2\xfc\xf8\xb4Pm;\xc4\x94}A~=R\xbcr\xbb?\xd1]\r\xb1\xd9{\xf6\x1b%\x14iAe\x04a\x91\x144q\x1e\x92H\xcb\xe7z,+1!6#\x92\x8c\x12o\x8eyb\xe7g\xd2[\x11W\xfeJ\xe3.\x88C\x1a$\xa5\xfa\xfd\xe1\x1e\x0c4\xc5\xbf7\x94\xca$\x0c\xa6\xbc\x07d\x04\x0f\xe4\xfc\xbeZ\x1c7\xce\x0c^8@d; \xf9\xfe\x1dU\x15\x9e\x9f[b\xe6Z\xda\xa9\x17\xcf\xd9\xa8\x0b\x10\xf5\xe3\xa1\xc0\xe2Z\x8b\x9fq\xe9\xe8\x97f\x1bY\xec\xbc\x8b\x89\x9a\xeb\xffU\xe2\xfa#%\xa5d\xfa\xeb\x15\"\x8a\xf4R\x85\xdf\xe3\xcd";

    #[test]
    fn should_connect_default_native_auth() -> anyhow::Result<()> {
        block_on(async {
            let mut mock = Mock::stream();

            mock.write_packet_async(0, SRV_HANDSHAKE_DEFAULT_NATIVE_AUTH).await?;
            mock.write_packet_async(2, SRV_AUTH_OK).await?;

            let _conn = MySqlConnectOptions::<Mock>::new()
                .port(mock.port())
                .username("root")
                .password("password")
                .connect()
                .await?;

            let buf = mock.read_all_async().await?;

            assert_eq!(&buf, RES_HANDSHAKE_NATIVE_AUTH);

            Ok(())
        })
    }

    #[test]
    fn should_connect_default_sha256_auth() -> anyhow::Result<()> {
        block_on(async {
            let mut mock = Mock::stream();

            mock.write_packet_async(0, SRV_HANDSHAKE_DEFAULT_SHA256_AUTH).await?;
            mock.write_packet_async(2, SRV_PUBLIC_KEY).await?;
            mock.write_packet_async(4, SRV_AUTH_OK).await?;

            let _conn = MySqlConnectOptions::<Mock>::new()
                .port(mock.port())
                .username("root")
                .password("password")
                .connect()
                .await?;

            let buf = mock.read_exact_async(RES_HANDSHAKE_SHA256_AUTH.len()).await?;
            assert_eq!(&buf, RES_HANDSHAKE_SHA256_AUTH);

            let buf = mock.read_all_async().await?;
            assert_eq!(&buf, RES_RSA_PASSWORD_SHA256);

            Ok(())
        })
    }

    #[test]
    fn should_connect_default_caching_sha2_auth() -> anyhow::Result<()> {
        block_on(async {
            let mut mock = Mock::stream();

            mock.write_packet_async(0, SRV_HANDSHAKE_DEFAULT_CACHING_SHA2_AUTH).await?;
            mock.write_packet_async(2, SRV_AUTH_MORE_CONTINUE).await?;
            mock.write_packet_async(4, SRV_PUBLIC_KEY).await?;
            mock.write_packet_async(6, SRV_AUTH_OK).await?;

            let _conn = MySqlConnectOptions::<Mock>::new()
                .port(mock.port())
                .username("root")
                .password("password")
                .connect()
                .await?;

            let buf = mock.read_exact_async(RES_HANDSHAKE_CACHING_SHA2_AUTH.len()).await?;
            assert_eq!(&buf, RES_HANDSHAKE_CACHING_SHA2_AUTH);

            let buf = mock.read_exact_async(RES_ASK_RSA_KEY.len()).await?;
            assert_eq!(&buf, RES_ASK_RSA_KEY);

            let buf = mock.read_all_async().await?;
            assert_eq!(&buf, RES_RSA_PASSWORD_CACHING_SHA2);

            Ok(())
        })
    }

    #[test]
    fn should_reconnect_default_caching_sha2_auth() -> anyhow::Result<()> {
        block_on(async {
            let mut mock = Mock::stream();

            mock.write_packet_async(0, SRV_HANDSHAKE_DEFAULT_CACHING_SHA2_AUTH).await?;
            mock.write_packet_async(2, SRV_AUTH_MORE_OK).await?;

            let _conn = MySqlConnectOptions::<Mock>::new()
                .port(mock.port())
                .username("root")
                .password("password")
                .connect()
                .await?;

            let buf = mock.read_all_async().await?;
            assert_eq!(&buf, RES_HANDSHAKE_CACHING_SHA2_AUTH);

            Ok(())
        })
    }

    #[test]
    fn should_connect_switch_native_auth() -> anyhow::Result<()> {
        block_on(async {
            let mut mock = Mock::stream();

            mock.write_packet_async(0, SRV_HANDSHAKE_DEFAULT_CACHING_SHA2_AUTH).await?;
            mock.write_packet_async(2, SRV_SWITCH_NATIVE_AUTH).await?;
            mock.write_packet_async(4, SRV_AUTH_OK).await?;

            let _conn = MySqlConnectOptions::<Mock>::new()
                .port(mock.port())
                .username("root")
                .password("password")
                .connect()
                .await?;

            let buf = mock.read_exact_async(RES_HANDSHAKE_CACHING_SHA2_AUTH.len()).await?;
            assert_eq!(&buf, RES_HANDSHAKE_CACHING_SHA2_AUTH);

            let buf = mock.read_all_async().await?;
            assert_eq!(&buf, RES_NATIVE_SCRAMBLE);

            Ok(())
        })
    }

    #[test]
    fn should_connect_switch_caching_sha2_auth() -> anyhow::Result<()> {
        block_on(async {
            let mut mock = Mock::stream();

            mock.write_packet_async(0, SRV_HANDSHAKE_DEFAULT_NATIVE_AUTH).await?;
            mock.write_packet_async(2, SRV_SWITCH_CACHING_SHA2_AUTH).await?;
            mock.write_packet_async(4, SRV_AUTH_MORE_CONTINUE).await?;
            mock.write_packet_async(6, SRV_PUBLIC_KEY).await?;
            mock.write_packet_async(8, SRV_AUTH_OK).await?;

            let _conn = MySqlConnectOptions::<Mock>::new()
                .port(mock.port())
                .username("root")
                .password("password")
                .connect()
                .await?;

            let buf = mock.read_exact_async(RES_HANDSHAKE_NATIVE_AUTH.len()).await?;
            assert_eq!(&buf, RES_HANDSHAKE_NATIVE_AUTH);

            let buf = mock.read_exact_async(RES_SHA_SCRAMBLE.len()).await?;
            assert_eq!(&buf, RES_SHA_SCRAMBLE);

            let buf = mock.read_exact_async(RES_ASK_RSA_KEY_2.len()).await?;
            assert_eq!(&buf, RES_ASK_RSA_KEY_2);

            let buf = mock.read_all_async().await?;
            assert_eq!(&buf, RES_SWITCH_RSA_PASSWORD_CACHING_SHA2);

            Ok(())
        })
    }

    #[test]
    fn should_reconnect_switch_caching_sha2_auth() -> anyhow::Result<()> {
        block_on(async {
            let mut mock = Mock::stream();

            mock.write_packet_async(0, SRV_HANDSHAKE_DEFAULT_NATIVE_AUTH).await?;
            mock.write_packet_async(2, SRV_SWITCH_CACHING_SHA2_AUTH).await?;
            mock.write_packet_async(4, SRV_AUTH_MORE_OK).await?;

            let _conn = MySqlConnectOptions::<Mock>::new()
                .port(mock.port())
                .username("root")
                .password("password")
                .connect()
                .await?;

            let buf = mock.read_exact_async(RES_HANDSHAKE_NATIVE_AUTH.len()).await?;
            assert_eq!(&buf, RES_HANDSHAKE_NATIVE_AUTH);

            let buf = mock.read_all_async().await?;
            assert_eq!(&buf, RES_SHA_SCRAMBLE);

            Ok(())
        })
    }

    #[test]
    fn should_connect_empty_auth() -> anyhow::Result<()> {
        block_on(async {
            let mut mock = Mock::stream();

            mock.write_packet_async(0, SRV_HANDSHAKE_DEFAULT_NATIVE_AUTH).await?;
            mock.write_packet_async(2, SRV_AUTH_OK).await?;

            let _conn = MySqlConnectOptions::<Mock>::new()
                .port(mock.port())
                .username("root")
                .connect()
                .await?;

            let buf = mock.read_all_async().await?;

            assert_eq!(&buf, RES_HANDSHAKE_EMPTY_AUTH);

            Ok(())
        })
    }

    #[test]
    fn should_not_connect_old_auth() -> anyhow::Result<()> {
        block_on(async {
            let mut mock = Mock::stream();

            mock.write_packet_async(0, SRV_HANDSHAKE_DEFAULT_OLD_AUTH).await?;

            let err = MySqlConnectOptions::<Mock>::new()
                .port(mock.port())
                .username("root")
                .password("password")
                .connect()
                .await
                .unwrap_err();

            assert_eq!(
                err.to_string(),
                "2059 (HY000): Authentication plugin 'mysql_old_password' cannot be loaded"
            );

            Ok(())
        })
    }
}
