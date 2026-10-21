# Manifest facts only: requirements are not resolved lockfile versions.
"| Package | Feature | Enables |", "| --- | --- | --- |",
(.packages | sort_by(.name)[] | .name as $package |
 .features | to_entries | sort_by(.key)[] |
 "| \($package) | \(.key) | \(.value | sort | join(", ")) |"),
"", "| Owner | Direct dependency | Requirement | Kind | Target | Optional | Defaults | Features |",
"| --- | --- | --- | --- | --- | --- | --- | --- |",
(.packages | sort_by(.name)[] | .name as $package |
 .dependencies | sort_by([.name, .kind, .target])[] |
 "| \($package) | \(.name) | \(if .path then "workspace-owned" else .req end) | \(.kind // "normal") | \(.target // "all") | \(.optional) | \(.uses_default_features) | \(.features | sort | join(", ")) |")
