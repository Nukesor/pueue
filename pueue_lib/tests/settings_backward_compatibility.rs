#[cfg(feature = "settings")]
mod tests {
    use color_eyre::{Result, eyre::WrapErr};
    use pueue_lib::Settings;
    use std::{path::PathBuf, vec};

    /// From 0.15.0 on, we aim to have full backward compatibility.
    /// For this reason, an old (slightly modified) v0.15.0 serialized settings file
    /// has been checked in.
    ///
    /// We have to be able to restore from that config at all costs.
    /// Everything else results in a breaking change and needs a major version change.
    /// (For `pueue_lib` as well as `pueue`!)
    ///
    /// On top of simply having old settings, I also removed a few default fields.
    /// This should be handled as well.
    #[test]
    fn test_restore_from_old_settings() -> Result<()> {
        better_panic::install();
        let old_settings_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests")
            .join("data")
            .join("v0.15.0_settings.yml");

        // Open v0.15.0 file and ensure the settings file can be read.
        let (settings, config_found) = Settings::read(&Some(old_settings_path))
            .wrap_err("Failed to read old config with defaults:")?;
        assert_eq!(settings.client.status.show_expanded_aliases, false);
        assert_eq!(settings.client.status.max_lines, Some(15));
        assert_eq!(settings.client.status.time_format, "mock015-%H:%M:%S");
        assert_eq!(
            settings.client.status.datetime_format,
            "mock015-%Y-%m-%d\n%H:%M:%S"
        );

        assert!(config_found);

        Ok(())
    }

    #[test]
    fn test_restore_from_4_0_2() -> Result<()> {
        let settings_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests")
            .join("data")
            .join("v4.0.2_settings.yml");

        // Open v4.0.2 file and ensure the settings file can be read.
        let (settings, config_found) = Settings::read(&Some(settings_path))
            .wrap_err("Failed to read old config with defaults:")?;
        assert_eq!(settings.client.status.show_expanded_aliases, true);
        assert_eq!(settings.client.status.max_lines, Some(42));
        assert_eq!(settings.client.status.time_format, "mock402-%H:%M:%S");
        assert_eq!(
            settings.client.status.datetime_format,
            "mock402-%Y-%m-%d\n%H:%M:%S"
        );
        assert_eq!(
            settings.client.status.additional_columns,
            vec!["Duration".to_owned(),]
        );

        assert!(config_found);

        Ok(())
    }
}
