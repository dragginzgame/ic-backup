# An isolated consumer may add its own root and prune unused packages, but may
# neither reselect a package from the original graph nor pull in the simulator.
def original_graph_admitted($original; $consumer):
    all(.packages[] | select(.name != $consumer);
        . as $package | any($original.packages[];
            .name == $package.name and .version == $package.version and .source == $package.source))
    and all(.packages[]; .name != "ic-testkit" and .name != "pocket-ic" and .name != "wat");

# Release qualification changes only the two workspace libraries to their exact
# crates.io identities. Every other selected package must keep its original source
# and version; all non-consumer packages must come from the registry.
def published_graph_admitted($original; $consumer):
    "registry+https://github.com/rust-lang/crates.io-index" as $registry
    | ($original | .packages |= map(
        if (.name == "ic-backup" or .name == "ic-backup-agent") and .source == null
        then .source = $registry else . end)) as $published
    | original_graph_admitted($published; $consumer)
    and all(.packages[] | select(.name != $consumer); .source == $registry)
    and ([.packages[] | select(.name == "ic-backup" or .name == "ic-backup-agent")] | length == 2);
