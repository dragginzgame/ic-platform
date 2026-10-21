# Map complete closure strings to collision-free, run-local short file IDs.
# Retain the original closure TSV separately for diagnostics and reproducibility.
BEGIN { FS = OFS = "\t" }
{
    if (NF != 2 || $1 == "" || $2 == "") {
        print "invalid feature closure mapping" > "/dev/stderr"
        exit 1
    }
    if (!($2 in ids)) ids[$2] = "graph-" ++count
    print $1, ids[$2]
}
