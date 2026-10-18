# Only local feature implication proves equivalence. Target-filtered optional
# dependencies are deliberately not collapsed: their exclusion must be tested.
def closure($features):
  . as $before |
  (. + [ .[] as $name | $features[$name][]? |
          select($features[.] != null) ]) | unique |
  if . == $before then . else closure($features) end;
($metadata[0].packages[] | select(.name == "ic-platform") | .features) as $features |
.[] as $selection |
(if $selection == "default" then "default"
 else (if $selection == "none" then [] else $selection | split(",") end |
       closure($features) | join(",")) end) as $key |
[$selection, (if $key == "" then "none" else $key end)] | @tsv
