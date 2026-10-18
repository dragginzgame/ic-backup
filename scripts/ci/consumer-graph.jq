# An isolated consumer may add its own root and prune unused packages, but may
# neither reselect a package from the original graph nor pull in the simulator.
def original_graph_admitted($original; $consumer):
    all(.packages[] | select(.name != $consumer);
        . as $package | any($original.packages[];
            .name == $package.name and .version == $package.version and .source == $package.source))
    and all(.packages[]; .name != "ic-testkit" and .name != "pocket-ic" and .name != "wat");
