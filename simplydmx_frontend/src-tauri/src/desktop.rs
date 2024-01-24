#![cfg_attr(
  all(not(debug_assertions), target_os = "windows"),
  windows_subsystem = "windows"
)]

pub fn main() {
  simplydmx::AppBuilder::new().setup(|_app| {
    if let Ok(mut nosleep) = nosleep::NoSleep::new() {
      let _ = nosleep.start(nosleep::NoSleepType::PreventUserIdleDisplaySleep);
    }
    #[cfg(target_os = "macos")]
    macos_app_nap::prevent();
    Ok(())
  }).run();
}
