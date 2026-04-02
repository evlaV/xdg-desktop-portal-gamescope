/*
 * Copyright © 2026 Valve Corporation
 *
 * SPDX-License-Identifier: BSD-3-Clause
 */

// Ensure all test functions are run sequentially, because each of them starts
// the portal backend, which claims a well-known name on the session bus.
// Running them in parallel would result in conflicts when claiming the name.
#[serial_test::serial]
mod integration_tests {

    use assert_cmd::cargo::CommandCargoExt;
    use assert_cmd::pkg_name;
    use futures_util::StreamExt;
    use std::collections::HashMap;
    use std::process::{Child, Command};
    use std::str::FromStr;
    use zbus::fdo::DBusProxy;
    use zbus::names::BusName;
    use zbus::zvariant::{Array, ObjectPath, Signature, Structure, Value};
    use zbus::{Connection, Proxy};

    include!(concat!(env!("CARGO_TARGET_DIR"), "/config.rs"));

    async fn start_portal_backend_daemon() -> Result<Child, Box<dyn std::error::Error>> {
        let cmd = Command::cargo_bin(pkg_name!())
            .unwrap()
            .spawn()
            .expect("failed to start portal backend");

        // Wait for the daemon to claim the well-known name on the session bus
        let connection = Connection::session().await?;
        let proxy = DBusProxy::new(&connection).await?;
        let mut name_owner_changed = proxy
            .receive_name_owner_changed_with_args(&[(0, BUSNAME)])
            .await?;
        name_owner_changed
            .next()
            .await
            .ok_or(zbus::names::Error::InvalidName(BUSNAME))?;
        assert!(
            proxy
                .name_has_owner(BusName::try_from(BUSNAME).unwrap())
                .await?
        );

        Ok(cmd)
    }

    async fn stop_portal_backend_daemon(cmd: &mut Child) -> Result<(), Box<dyn std::error::Error>> {
        cmd.kill().expect("failed to stop portal backend");

        // Wait for the well-known name to be released on the session bus
        let connection = Connection::session().await?;
        let proxy = DBusProxy::new(&connection).await?;
        assert!(
            !proxy
                .name_has_owner(BusName::try_from(BUSNAME).unwrap())
                .await?
        );

        Ok(())
    }

    #[tokio::test]
    async fn access_portal_backend() -> Result<(), Box<dyn std::error::Error>> {
        // Start the portal backend daemon
        let mut cmd = start_portal_backend_daemon().await?;

        // Test the Access portal
        let connection = Connection::session().await?;
        let proxy = Proxy::new(
            &connection,
            BUSNAME,
            "/org/freedesktop/portal/desktop",
            "org.freedesktop.impl.portal.Access",
        )
        .await?;
        let mut options = HashMap::<&str, Value>::new();
        let mut choices = Array::new(&Signature::from_str("ssa(ss)s").unwrap());
        let mut encodings = vec![];
        encodings.push(("none", "No encoding"));
        encodings.push(("utf8", "Unicode (UTF-8)"));
        encodings.push(("latin15", "Western"));
        choices.append(Value::new(("encoding", "Encoding", encodings, "utf8")))?;
        options.insert("choices", Value::new(choices));
        let m = proxy
            .call_method(
                "AccessDialog",
                &(
                    ObjectPath::from_static_str_unchecked("/o/f/p/d/request/3_14/foo"),
                    "org.example.Test",
                    "",
                    "title",
                    "subtitle",
                    "body",
                    options,
                ),
            )
            .await?;
        let body = m.body();
        let (response, results) = body.deserialize::<(u32, HashMap<&str, Value>)>().unwrap();
        assert_eq!(response, 0);
        assert!(results.contains_key("choices"));
        let choices = Array::try_from(results.get("choices").unwrap()).unwrap();
        let fields = Structure::try_from(choices.first().unwrap()).unwrap();
        assert_eq!(
            fields.fields(),
            [Value::from("encoding"), Value::from("utf8")]
        );

        // Terminate the portal backend daemon
        Ok(stop_portal_backend_daemon(&mut cmd).await?)
    }

    #[tokio::test]
    async fn screencast_portal_backend() -> Result<(), Box<dyn std::error::Error>> {
        // Start the portal backend daemon
        let mut cmd = start_portal_backend_daemon().await?;

        // Test the ScreenCast portal
        let connection = Connection::session().await?;
        let proxy = Proxy::new(
            &connection,
            BUSNAME,
            "/org/freedesktop/portal/desktop",
            "org.freedesktop.impl.portal.ScreenCast",
        )
        .await?;

        // Test the values of various properties
        assert_eq!(
            proxy.get_property::<u32>("AvailableSourceTypes").await,
            Ok(3)
        );
        assert_eq!(
            proxy.get_property::<u32>("AvailableCursorModes").await,
            Ok(7)
        );

        // Create a session
        let m = proxy
            .call_method(
                "CreateSession",
                &(
                    ObjectPath::from_static_str_unchecked("/o/f/p/d/request/3_14/foo"),
                    ObjectPath::from_static_str_unchecked("/o/f/p/d/session/3_14/bar"),
                    "org.example.Test",
                    HashMap::<&str, Value>::new(),
                ),
            )
            .await?;
        let body = m.body();
        let (response, results) = body.deserialize::<(u32, HashMap<&str, Value>)>().unwrap();
        assert_eq!(response, 0);
        assert_eq!(results.get("session_id"), Some(&Value::from("bar")));

        // Start the session
        // TODO: this would require mocking the custom gamescope pipewire protocol
        // over the gamescope wayland socket (${XDG_RUNTIME_DIR}/gamescope-0)

        // Close the session
        let session_proxy = Proxy::new(
            &connection,
            BUSNAME,
            "/o/f/p/d/session/3_14/bar",
            "org.freedesktop.impl.portal.Session",
        )
        .await?;
        session_proxy
            .call_noreply("Close", &())
            .await
            .expect("failed to close session");

        // Terminate the portal backend daemon
        Ok(stop_portal_backend_daemon(&mut cmd).await?)
    }

    #[tokio::test]
    async fn screenshot_portal_backend() -> Result<(), Box<dyn std::error::Error>> {
        // Start the portal backend daemon
        let mut cmd = start_portal_backend_daemon().await?;

        // Test the Screenshot portal
        let connection = Connection::session().await?;
        let proxy = Proxy::new(
            &connection,
            BUSNAME,
            "/org/freedesktop/portal/desktop",
            "org.freedesktop.impl.portal.Screenshot",
        )
        .await?;
        let m = proxy
            .call_method(
                "Screenshot",
                &(
                    ObjectPath::from_static_str_unchecked("/o/f/p/d/request/3_14/foo"),
                    "org.example.Test",
                    "",
                    HashMap::<&str, Value>::new(),
                ),
            )
            .await?;
        let body = m.body();
        let (response, results) = body.deserialize::<(u32, HashMap<&str, Value>)>().unwrap();
        assert_eq!(response, 0);
        assert!(results.contains_key("uri"));
        let uri = url::Url::parse(&String::try_from(results.get("uri").unwrap()).unwrap())?;
        assert!(uri.to_file_path().unwrap().try_exists()?);

        // Terminate the portal backend daemon
        Ok(stop_portal_backend_daemon(&mut cmd).await?)
    }
}
