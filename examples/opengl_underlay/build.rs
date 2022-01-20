// Copyright © SixtyFPS GmbH <info@sixtyfps.io>
// SPDX-License-Identifier: (GPL-3.0-only OR LicenseRef-SixtyFPS-commercial)

fn main() {
    let config = sixtyfps_build::CompilerConfiguration::default().with_style("fluent".into());
    sixtyfps_build::compile_with_config("scene.60", config).unwrap();
}
