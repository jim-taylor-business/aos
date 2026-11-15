use leptos_i18n_build::{Config, ParseOptions, TranslationsInfos};
use std::{error::Error, path::PathBuf};

fn main() -> Result<(), Box<dyn Error>> {
  println!("cargo::rerun-if-changed=build.rs");
  println!("cargo::rerun-if-changed=Cargo.toml");

  let output_dir = PathBuf::from(std::env::var_os("OUT_DIR").unwrap()).join("i18n");
  let config = Config::new("en")?.add_locale("fr")?.parse_options(ParseOptions::default().interpolate_display(true));
  let translations = TranslationsInfos::parse(config)?;
  translations.emit_diagnostics();
  translations.rerun_if_locales_changed();
  translations.generate_i18n_module(output_dir)?;

  Ok(())
}
